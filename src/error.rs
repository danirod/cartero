use srtemplate::SrTemplateError;
use thiserror::Error;

use crate::client::RequestError;

#[derive(Debug, Error)]
pub enum CarteroError {
    #[error("Internal error")]
    InternalError,

    #[error("No file has been picked")]
    NoFilePicked,

    #[error("Internal error on file dialog")]
    FileDialogError,

    #[error("Not a collection")]
    NotValidCollection,

    #[error("Collection already opened")]
    AlreadyOpened,

    #[error("DNS error")]
    Dns,

    #[error("HTTP request error")]
    Request(#[from] RequestError),

    #[error("Error operating with files")]
    FileError(#[from] std::io::Error),

    #[error("Error manipulating TOML")]
    DeserializationError(#[from] toml::de::Error),

    #[error("Error manipulating TOML")]
    SerializationError(#[from] toml::ser::Error),

    #[error("Outdated schema, please update the software")]
    OutdatedSchema,

    #[error("{0}")]
    PreconditionError(#[from] RequestPreconditionError),
}

#[derive(Debug, Eq, PartialEq, Error)]
pub enum RequestPreconditionError {
    #[error("Cannot parse the URL, check for typos")]
    UrlBadParse,

    #[error("URL is missing a protocol")]
    MissingProtocol,

    #[error("Protocol {0}:// is not supported")]
    UnsupportedProtocol(String),

    #[error("Payload could not be encoded")]
    EncodingError,

    #[error("Variable {0} not found")]
    VariableNotFound(String),

    #[error("String interpolation error, review variables")]
    BadInterpolation,
}

impl From<SrTemplateError> for RequestPreconditionError {
    fn from(value: SrTemplateError) -> Self {
        match value {
            SrTemplateError::VariableNotFound(var) => Self::VariableNotFound(var),
            _ => Self::BadInterpolation,
        }
    }
}
