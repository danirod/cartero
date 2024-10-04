use srtemplate::SrTemplateError;
use thiserror::Error;

use crate::client::RequestError;

#[derive(Debug, Error)]
pub enum CarteroError {
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

    #[error("Invalid protocol")]
    InvalidProtocol,

    #[error("HTTP request error")]
    Request(#[from] RequestError),

    #[error("Error operating with files")]
    FileError(#[from] std::io::Error),

    #[error("Error manipulating TOML")]
    DeserializationError(#[from] toml::de::Error),

    #[error("Error manipulating TOML")]
    SerializationError(#[from] toml::ser::Error),

    #[error("Error during variable interpolation: {0}")]
    VariableInterpolationError(#[from] SrTemplateError),

    #[error("Outdated schema, please update the software")]
    OutdatedSchema,
}
