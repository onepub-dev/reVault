use std::fmt;
use std::sync::PoisonError;

use crate::payload;
use crate::protocol::{Operation, ProtocolError, Status};

#[derive(Debug)]
/// Represents client error.
pub enum ClientError {
    /// Represents the io case.
    Io(std::io::Error),
    /// Represents the url case.
    Url(String),
    /// Represents the http case.
    Http(String),
    /// Represents the protocol case.
    Protocol(ProtocolError),
    /// Represents the payload case.
    Payload(payload::PayloadError),
    /// Represents the topology case.
    Topology(String),
    /// Represents the replication case.
    Replication(String),
    /// Represents the server case.
    Server {
        /// Represents the status carried by this record case.
        status: Status,
        /// Represents the message carried by this record case.
        message: String,
    },
    /// Represents the unexpected operation case.
    UnexpectedOperation {
        /// Represents the expected carried by this record case.
        expected: Operation,
        /// Represents the actual carried by this record case.
        actual: Operation,
    },
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Url(err) => write!(f, "invalid key server url: {err}"),
            Self::Http(err) => write!(f, "http error: {err}"),
            Self::Protocol(err) => write!(f, "protocol error: {err}"),
            Self::Payload(err) => write!(f, "payload error: {err}"),
            Self::Topology(err) => write!(f, "topology error: {err}"),
            Self::Replication(err) => write!(f, "replication error: {err}"),
            Self::Server { status, message } => write!(f, "server returned {status:?}: {message}"),
            Self::UnexpectedOperation { expected, actual } => {
                write!(f, "expected {expected:?} response, got {actual:?}")
            }
        }
    }
}

impl std::error::Error for ClientError {}

impl From<std::io::Error> for ClientError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ProtocolError> for ClientError {
    fn from(value: ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

impl From<payload::PayloadError> for ClientError {
    fn from(value: payload::PayloadError) -> Self {
        Self::Payload(value)
    }
}

pub(crate) fn publish_state_poisoned<T>(_err: PoisonError<T>) -> ClientError {
    ClientError::Topology("publish client state lock was poisoned".to_string())
}
