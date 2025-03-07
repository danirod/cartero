use gettextrs::gettext;
use srtemplate::SrTemplateError;
use thiserror::Error;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileLoadError {
    AnonymousPane,
    FileReadError(glib::Error),
    DeserializationError(toml::de::Error),
    OutdatedSchema,
}

impl std::fmt::Display for FileLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::AnonymousPane => gettext("The current tab is not associated with a file"),
            Self::OutdatedSchema => gettext(
                "This file was created with a newer version of this application; please update!",
            ),
            Self::FileReadError(e) => e.message().to_string(),
            Self::DeserializationError(_) => {
                gettext("The file is corrupt or does not contain valid data for this application")
            }
        };
        write!(f, "{}", message)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileSaveError {
    #[allow(dead_code)]
    AnonymousPane,
    FileWriteError(glib::Error),
    SerializationError(toml::ser::Error),
}

impl std::fmt::Display for FileSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::AnonymousPane => gettext("The current tab is not associated with a file"),
            Self::FileWriteError(e) => e.message().to_string(),
            Self::SerializationError(e) => e.to_string(),
        };
        write!(f, "{}", message)
    }
}
