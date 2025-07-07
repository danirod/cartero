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

use cartero_interop::{FileLoadError, FileOpResult, FileSaveError};
use cartero_objects::{Field, Request};

mod authorization_value;
mod field_table_value;
mod field_value;
mod payload_value;
mod query_value;
mod request_value;
mod serializer;

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
pub fn deserialize_request(input: impl AsRef<str>) -> Result<FileOpResult<Request>, FileLoadError> {
    let value = toml::from_str::<request_value::RequestValue>(input.as_ref());
    match value {
        Ok(value) => value.try_into(),
        Err(e) => Err(FileLoadError::DeserializationError(Box::new(e))),
    }
}

pub fn serialize_request(req: &Request) -> Result<String, FileSaveError> {
    let value = request_value::RequestValue::from(req.clone());
    crate::serializer::serialize(&value)
}

#[cfg(test)]
mod tests {
    use crate::{deserialize_request, serialize_request};
    use cartero_objects::{Field, Request, RequestBodyMultipart, RequestBodyType, RequestMethod};

    #[test]
    fn deserialize_trash() {
        let result = deserialize_request("hello");
        assert!(result.is_err());
    }

    #[test]
    fn serialize_req() {
        let req: Request = glib::Object::builder()
            .property("url", "https://www.example.com/test.html")
            .property("method", RequestMethod::Get)
            .build();
        let encoded = serialize_request(&req).unwrap();
        assert!(encoded.contains("url = \"https://www.example.com/test.html"));
        assert!(encoded.contains("method = \"GET\""));
    }

    // This test mostly checks that inline tables and non-inline tables are
    // used successfully. Variables should be encoded as an inline table,
    // period. For deeper serialization tests, check the integration tests.
    #[test]
    fn serialize_req_with_param_table() {
        let req: Request = glib::Object::builder()
            .property("url", "https://www.example.com/api/foo")
            .property("method", RequestMethod::Post)
            .build();
        let field = Field::from(("foo", "bar"));
        field.set_active(false);
        field.set_masked(true);
        let multipart = RequestBodyMultipart::new();
        multipart.params().insert(&field);
        req.body().set_body_type(RequestBodyType::Multipart);
        req.body().set_body_data(Some(multipart.as_ref()));

        let encoded = serialize_request(&req).unwrap();
        assert!(!encoded.contains("[body.variables.foo]"));
        assert!(encoded.contains("foo = { value = \"bar\", active = false, secret = true"));
    }
}
