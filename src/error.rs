use crate::i18n::i18n_f;
use gettextrs::gettext;
use srtemplate::SrTemplateError;

#[derive(Debug, Eq, PartialEq)]
pub enum RequestPreconditionError {
    UrlBadParse,
    MissingProtocol,
    UnsupportedProtocol(String),
    EncodingError,
    VariableNotFound(String),
    BadInterpolation,
}

impl std::fmt::Display for RequestPreconditionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UrlBadParse => gettext("Cannot recognise the URL"),
            Self::MissingProtocol => gettext("The given URL is missing a protocol"),
            Self::UnsupportedProtocol(proto) => {
                i18n_f("The protocol {}:// is not supported", &[&proto])
            }
            Self::EncodingError => gettext("The given request body could not be encoded correctly"),
            Self::VariableNotFound(var) => i18n_f("The variable '{}' is not defined", &[&var]),
            Self::BadInterpolation => {
                gettext("There was a problem with a variable interpolation, review your inputs")
            }
        };
        write!(f, "{}", message)
    }
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
