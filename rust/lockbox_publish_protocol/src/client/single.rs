use std::thread;
use std::time::Duration;

use crate::client::{
    ClientError, ContactPublish, HttpTransport, PublishClient, PublishResult, ReceivedPublish,
    TopologyAwareResponse, Transport, DEFAULT_MAX_RESPONSE_BYTES,
};
use crate::payload;
use crate::protocol::{self, Operation, Status};

use super::helpers::{next_backoff, retry_single_client_error, topology_from_tail};
use super::types::RetryPolicy;

impl PublishClient<HttpTransport> {
    pub fn new(server_url: &str) -> Result<Self, ClientError> {
        Ok(Self {
            transport: HttpTransport::new(server_url)?,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            retry_policy: RetryPolicy::default(),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.transport.timeout = timeout;
        self
    }
}

impl<T: Transport> PublishClient<T> {
    pub fn from_transport(transport: T) -> Self {
        Self::new_inner(transport)
    }

    pub fn with_max_response_bytes(mut self, max_response_bytes: usize) -> Self {
        self.max_response_bytes = max_response_bytes;
        self
    }

    pub fn with_retry_policy(
        mut self,
        attempts: usize,
        initial_backoff: Duration,
        max_backoff: Duration,
    ) -> Self {
        self.retry_policy = RetryPolicy {
            attempts: attempts.max(1),
            initial_backoff,
            max_backoff,
        };
        self
    }

    pub fn publish_payload(
        &self,
        ttl_seconds: u32,
        max_receives: u16,
        payload: &[u8],
    ) -> Result<PublishResult, ClientError> {
        self.publish_payload_with_email(ttl_seconds, max_receives, payload, None)
    }

    pub fn publish_payload_with_email(
        &self,
        ttl_seconds: u32,
        max_receives: u16,
        payload: &[u8],
        verification_email: Option<&str>,
    ) -> Result<PublishResult, ClientError> {
        Ok(self
            .publish_payload_with_email_with_version(
                ttl_seconds,
                max_receives,
                payload,
                verification_email,
                None,
            )?
            .value)
    }

    pub fn publish_payload_with_email_with_version(
        &self,
        ttl_seconds: u32,
        max_receives: u16,
        payload: &[u8],
        verification_email: Option<&str>,
        topology_version: Option<u64>,
    ) -> Result<TopologyAwareResponse<PublishResult>, ClientError> {
        payload::validate_payload(payload)?;
        let body = protocol::encode_publish_request_with_email(
            ttl_seconds,
            max_receives,
            payload,
            verification_email,
        );
        let response = self.request_with_retry(
            Operation::Publish,
            &body,
            topology_version,
            retry_single_client_error,
        )?;
        let decoded = protocol::decode_publish_response_document(&response.value.payload)?;
        Ok(TopologyAwareResponse {
            value: PublishResult {
                publish_code: decoded.publish_code,
                delete_token: decoded.delete_token,
                expires_at_unix_ms: decoded.expires_at_unix_ms,
                max_receives: decoded.max_receives,
                verification_url: decoded.verification_url,
            },
            topology: response.topology,
        })
    }

    pub fn publish_contact(
        &self,
        ttl_seconds: u32,
        max_receives: u16,
        contact: ContactPublish<'_>,
    ) -> Result<PublishResult, ClientError> {
        let payload = payload::encode_contact_publish(
            contact.identity,
            contact.public_key,
            contact.signing_public_key,
            contact.fingerprint,
            contact.publish_nonce,
            contact.created_at_unix_ms,
            contact.expires_at_unix_ms,
        );
        self.publish_payload_with_email(
            ttl_seconds,
            max_receives,
            &payload,
            contact.verification_email,
        )
    }

    pub fn receive(&self, publish_code: &str) -> Result<ReceivedPublish, ClientError> {
        Ok(self.receive_with_version(publish_code, None)?.value)
    }

    pub fn receive_with_version(
        &self,
        publish_code: &str,
        topology_version: Option<u64>,
    ) -> Result<TopologyAwareResponse<ReceivedPublish>, ClientError> {
        let body = protocol::encode_receive_request(publish_code);
        let response = self.request_with_retry(
            Operation::Receive,
            &body,
            topology_version,
            retry_single_client_error,
        )?;
        let decoded = protocol::decode_receive_response_document(&response.value.payload)?;
        let payload_type = payload::validate_payload(&decoded.publish_payload)?;
        Ok(TopologyAwareResponse {
            value: ReceivedPublish {
                payload: decoded.publish_payload,
                payload_type,
                expires_at_unix_ms: decoded.expires_at_unix_ms,
                remaining_receives: decoded.remaining_receives,
                email_verification: decoded.email_verification,
            },
            topology: response.topology,
        })
    }

    pub fn delete(&self, publish_code: &str, delete_token: &[u8]) -> Result<bool, ClientError> {
        Ok(self
            .delete_with_version(publish_code, delete_token, None)?
            .value)
    }

    pub fn delete_with_version(
        &self,
        publish_code: &str,
        delete_token: &[u8],
        topology_version: Option<u64>,
    ) -> Result<TopologyAwareResponse<bool>, ClientError> {
        let body = protocol::encode_delete_request(publish_code, delete_token);
        let response = self.request_with_retry(
            Operation::Delete,
            &body,
            topology_version,
            retry_single_client_error,
        )?;
        Ok(TopologyAwareResponse {
            value: protocol::decode_delete_response(&response.value.payload)
                .map_err(ClientError::from)?,
            topology: response.topology,
        })
    }

    fn request_with_retry(
        &self,
        expected: Operation,
        body: &[u8],
        topology_version: Option<u64>,
        retry: impl Fn(&ClientError) -> bool,
    ) -> Result<TopologyAwareResponse<protocol::ResponseEnvelope>, ClientError> {
        let attempts = self.retry_policy.attempts.max(1);
        let mut backoff = self.retry_policy.initial_backoff;
        let mut last_error = None;
        for attempt in 0..attempts {
            match self
                .transport
                .post_binary_with_topology(body, topology_version)
                .and_then(|response| self.success_response_with_topology(expected, &response))
            {
                Ok(response) => return Ok(response),
                Err(err) if retry(&err) && attempt + 1 < attempts => {
                    last_error = Some(err);
                    if !backoff.is_zero() {
                        thread::sleep(backoff);
                    }
                    backoff = next_backoff(backoff, self.retry_policy.max_backoff);
                }
                Err(err) => return Err(err),
            }
        }
        Err(last_error
            .unwrap_or_else(|| ClientError::Url("retry policy has no attempts".to_string())))
    }

    fn success_response_with_topology(
        &self,
        expected: Operation,
        bytes: &[u8],
    ) -> Result<TopologyAwareResponse<protocol::ResponseEnvelope>, ClientError> {
        let response_with_tail =
            protocol::decode_response_with_tail(bytes, self.max_response_bytes)?;
        let response = response_with_tail.envelope;
        if response.operation != expected {
            return Err(ClientError::UnexpectedOperation {
                expected,
                actual: response.operation,
            });
        }
        if response.status != Status::Success {
            let message = protocol::decode_error_payload(&response.payload)
                .map(|(_, message)| message)
                .unwrap_or_else(|err| err.to_string());
            return Err(ClientError::Server {
                status: response.status,
                message,
            });
        }
        Ok(TopologyAwareResponse {
            value: response,
            topology: topology_from_tail(&response_with_tail.tail),
        })
    }
}
