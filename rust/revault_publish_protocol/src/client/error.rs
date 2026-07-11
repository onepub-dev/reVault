use std::fmt;
use std::sync::PoisonError;

use crate::payload;
use crate::protocol::{Operation, ProtocolError, Status};

#[derive(Debug)]
pub enum ClientError {
    Io(std::io::Error),
    Url(String),
    Http(String),
    Protocol(ProtocolError),
    Payload(payload::PayloadError),
    Topology(String),
    Replication(String),
    Server {
        status: Status,
        message: String,
    },
    UnexpectedOperation {
        expected: Operation,
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
