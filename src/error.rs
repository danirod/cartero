// Copyright 2024-2025 the Cartero authors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use formatx::formatx;
use gettextrs::gettext;

#[derive(Debug, Eq, PartialEq)]
pub enum RequestBuildError {
    InvalidUrl(url::ParseError),
    InvalidHeaderName(String),
    InvalidHeaderValue(String),
    InvalidBodyEncoding,
}

impl std::fmt::Display for RequestBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidUrl(_) => gettext("The specified URL is not valid"),
            Self::InvalidHeaderName(name) => {
                formatx!(gettext("The header '{}' is not valid"), name).unwrap()
            }
            Self::InvalidHeaderValue(name) => {
                formatx!(gettext("The value for header '{}' is not valid"), name).unwrap()
            }
            Self::InvalidBodyEncoding => {
                gettext("The given request body could not be encoded correctly")
            }
        };
        write!(f, "{}", message)
    }
}

impl std::error::Error for RequestBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUrl(pe) => Some(pe),
            _ => None,
        }
    }
}

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
                formatx!(gettext("The protocol {}:// is not supported"), proto).unwrap()
            }
            Self::EncodingError => gettext("The given request body could not be encoded correctly"),
            Self::VariableNotFound(var) => {
                formatx!(gettext("The variable '{}' is not defined"), var).unwrap()
            }
            Self::BadInterpolation => {
                gettext("There was a problem with a variable interpolation, review your inputs")
            }
        };
        write!(f, "{}", message)
    }
}

impl From<srtemplate::Error> for RequestPreconditionError {
    fn from(value: srtemplate::Error) -> Self {
        match value {
            srtemplate::Error::VariableNotFound(var) => Self::VariableNotFound(var),
            _ => Self::BadInterpolation,
        }
    }
}

#[derive(Debug)]
pub enum RequestError {
    NetworkError(isahc::error::Error),
    IOError(std::io::Error),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NetworkError(_) => gettext("There is a network error"),
            Self::IOError(_) => gettext("There is an input/output error"),
        };
        write!(f, "{}", message)
    }
}

impl std::error::Error for RequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NetworkError(e) => Some(e),
            Self::IOError(e) => Some(e),
        }
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
