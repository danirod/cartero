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

use cartero_http::RequestError;
use cartero_interop::{FileLoadError, FileSaveError, FileWarningTag};
use formatx::formatx;
use gettextrs::gettext;

pub enum InnerError<T> {
    InteropError(T),
    GlibError(glib::Error),
}

impl std::fmt::Display for InnerError<FileLoadError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InteropError(fe) => match fe {
                FileLoadError::SchemaTooNew => gettext("This file was created with a newer version of this application; please update!"),
                FileLoadError::DeserializationError(_) => gettext("The file is corrupt or does not contain valid data for this application")
            },
            Self::GlibError(e) => e.message().to_string(),
        };
        write!(f, "{}", message)
    }
}

impl std::fmt::Display for InnerError<FileSaveError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InteropError(fe) => match fe {
                FileSaveError::SerializationError(e) => e.to_string(),
            },
            Self::GlibError(e) => e.message().to_string(),
        };
        write!(f, "{}", message)
    }
}

pub fn get_request_error_message(error: &RequestError) -> String {
    match error {
        RequestError::EmptyUrl => gettext("The specified URL is not valid"),
        RequestError::UrlBadParse => gettext("Cannot recognise the URL"),
        RequestError::MissingProtocol => gettext("The given URL is missing a protocol"),
        RequestError::UnsupportedProtocol(proto) => {
            formatx!(gettext("The protocol {}:// is not supported"), proto).unwrap()
        }
        RequestError::VariableNotFound(var) => {
            formatx!(gettext("The variable '{}' is not defined"), var).unwrap()
        }
        RequestError::BadInterpolation => {
            gettext("There was a problem with a variable interpolation, review your inputs")
        }
        RequestError::InvalidHeaderName(name) => {
            formatx!(gettext("The header '{}' is not valid"), name).unwrap()
        }
        RequestError::InvalidHeaderValue(name) => {
            formatx!(gettext("The value for header '{}' is not valid"), name).unwrap()
        }
        RequestError::EncodingError => {
            gettext("The given request body could not be encoded correctly")
        }
        RequestError::FilePrefixUnset => gettext("You have to save the request first"),
        RequestError::UnsecureFile(str) => formatx!(
            gettext("The file '{}' cannot be accessed due to the security policy"),
            str
        )
        .unwrap(),
        RequestError::IOError(_) => gettext("There is an input/output error"),
        RequestError::NetworkError(_) => gettext("There is a network error"),
    }
}

pub enum LoadResult {
    Successful,
    Anonymous,
    Warning(Vec<FileWarningTag>),
    Error(InnerError<FileLoadError>),
}

pub enum SaveResult {
    Successful,
    Anonymous,
    Error(InnerError<FileSaveError>),
}
