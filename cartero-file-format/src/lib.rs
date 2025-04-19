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

#![doc = include_str!("../README.md")]

use cartero_interop::{FileLoadError, FileLoadResult};
use cartero_objects::{Field, Request};

mod authorization_value;
mod field_table_value;
mod field_value;
mod payload_value;
mod query_value;
mod request_value;

pub(crate) trait ToField {
    fn to_field(&self, key: &str) -> Field;
}

/// Given the string representation of a request (in TOML format), this function
/// will try to decode it into a [`Request`] object.
///
/// The result of this function can actually encode three possible values:
/// - A non-recoverable error, which means that the given data cannot be loaded.
///   Examples are TOML deserialization errors or an invalid version number.
/// - A file loaded successfully, which means that it's completely valid.
/// - A file loaded with warnings, which means that the program could recover
///   despite the file having some issues.
///
/// In case of error, this is received by the Err() part of the given Result.
/// The Result returns Ok() if the file can be loaded, and any possible
/// warnings can be checked from the [`FileLoadResult`] object, if apply.
pub fn deserialize_request(
    input: impl AsRef<str>,
) -> Result<FileLoadResult<Request>, FileLoadError> {
    let value = toml::from_str::<request_value::RequestValue>(input.as_ref());
    match value {
        Ok(value) => value.try_into(),
        Err(e) => Err(FileLoadError::DeserializationError(Box::new(e))),
    }
}

#[cfg(test)]
mod tests {
    use crate::deserialize_request;

    #[test]
    fn deserialize_trash() {
        let result = deserialize_request("hello");
        assert!(result.is_err());
    }
}
