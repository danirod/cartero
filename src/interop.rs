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

use cartero_interop::{FileLoadError, FileSaveError, FileWarningTag};
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
