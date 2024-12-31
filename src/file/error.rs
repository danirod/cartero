// Copyright 2024 the Cartero authors
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

use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FileError {
    #[error("Object has not been assigned a path")]
    NoPath,

    #[error("File cannot be read")]
    CannotRead,

    #[error("File cannot be parsed as a valid local object")]
    CannotParseFile,

    #[error("Payload cannot be encoded")]
    CannotEncodePayload,

    #[error("File cannot be saved")]
    CannotSave,
}
