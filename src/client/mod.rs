mod http_client;
mod models;

use std::error::Error;

use isahc::http::header::{InvalidHeaderName, InvalidHeaderValue};
pub use models::{Request, RequestMethod, Response};

#[derive(Debug)]
pub struct RequestError;

impl Error for RequestError {}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request Error")
    }
}

impl From<InvalidHeaderName> for RequestError {
    fn from(_: InvalidHeaderName) -> Self {
        Self
    }
}

impl From<InvalidHeaderValue> for RequestError {
    fn from(_: InvalidHeaderValue) -> Self {
        Self
    }
}

impl From<&str> for RequestError {
    fn from(_: &str) -> Self {
        Self
    }
}

impl From<isahc::http::Error> for RequestError {
    fn from(_: isahc::http::Error) -> Self {
        Self
    }
}

impl From<std::io::Error> for RequestError {
    fn from(_: std::io::Error) -> Self {
        Self
    }
}
