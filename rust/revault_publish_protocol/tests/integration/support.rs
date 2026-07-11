use std::cell::RefCell;
use std::time::Duration;

use lockbox_publish_protocol::client::{PublishClient, Transport};
use lockbox_publish_protocol::protocol::{self, Operation, Status};

#[derive(Clone)]
pub struct MockTransport {
    responses: std::rc::Rc<RefCell<Vec<Vec<u8>>>>,
    calls: std::rc::Rc<RefCell<usize>>,
}

#[derive(Clone)]
pub struct FlakyTransport {
    response: Vec<u8>,
    calls: std::rc::Rc<RefCell<usize>>,
}

impl FlakyTransport {
    pub fn new(response: Vec<u8>) -> Self {
        Self {
            response,
            calls: std::rc::Rc::new(RefCell::new(0)),
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.borrow()
    }
}

impl Transport for FlakyTransport {
    fn post_binary(&self, _body: &[u8]) -> Result<Vec<u8>, lockbox_publish_protocol::ClientError> {
        let mut calls = self.calls.borrow_mut();
        *calls += 1;
        if *calls == 1 {
            return Err(lockbox_publish_protocol::ClientError::Io(
                std::io::ErrorKind::WouldBlock.into(),
            ));
        }
        Ok(self.response.clone())
    }
}

impl MockTransport {
    pub fn new(responses: Vec<Vec<u8>>) -> Self {
        Self {
            responses: std::rc::Rc::new(RefCell::new(responses)),
            calls: std::rc::Rc::new(RefCell::new(0)),
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.borrow()
    }
}

impl Transport for MockTransport {
    fn post_binary(&self, _body: &[u8]) -> Result<Vec<u8>, lockbox_publish_protocol::ClientError> {
        *self.calls.borrow_mut() += 1;
        Ok(self.responses.borrow_mut().remove(0))
    }
}

pub fn contact_publish_payload() -> Vec<u8> {
    lockbox_publish_protocol::encode_contact_publish(
        "client@example.com",
        b"public-key-material",
        b"signing-public-key-material",
        &[1_u8; 32],
        &[2_u8; 24],
        1,
        2,
    )
}

pub fn publish_success_response(publish_code: &str) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    protocol::put_string(&mut body, publish_code);
    protocol::put_bytes(&mut body, b"delete-token");
    protocol::put_u64(&mut body, 2);
    protocol::put_u16(&mut body, 1);
    protocol::encode_response(Operation::Publish, Status::Success, &body)
}

pub fn receive_success_response(payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    protocol::put_bytes(&mut body, payload);
    protocol::put_u64(&mut body, 2);
    protocol::put_u16(&mut body, 0);
    protocol::encode_response(Operation::Receive, Status::Success, &body)
}

pub fn delete_success_response(deleted: bool) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    body.push(u8::from(deleted));
    protocol::encode_response(Operation::Delete, Status::Success, &body)
}

pub fn rate_limited_response(operation: Operation) -> Vec<u8> {
    let mut body = Vec::new();
    protocol::put_u16(&mut body, protocol::MESSAGE_VERSION);
    protocol::put_u16(&mut body, Status::RateLimited as u16);
    protocol::put_string(&mut body, "rate limited");
    protocol::encode_response(operation, Status::RateLimited, &body)
}

pub fn one_attempt_client(transport: MockTransport) -> PublishClient<MockTransport> {
    PublishClient::from_transport(transport).with_retry_policy(1, Duration::ZERO, Duration::ZERO)
}
