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

use srtemplate::SrTemplateError;
use thiserror::Error;

use crate::client::RequestError;

#[derive(Debug, Error)]
pub enum CarteroError {
    #[error("No file has been picked")]
    NoFilePicked,

    #[error("Internal error on file dialog")]
    FileDialogError,

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

#[derive(Debug, Error)]
pub enum FileOperationError {
    #[error("No file was assigned")]
    NoFileGiven,

    #[error("Cannot read file")]
    FileReadError,

    #[error("Cannot decode file")]
    FileDecodeError,

    #[error("Cannot encode file")]
    FileEncodeError,

    #[error("Cannot write file")]
    FileWriteError,
}
