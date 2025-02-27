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

use std::io::{BufWriter, Write};

use crate::{
    entities::{EndpointData, RawEncoding, RequestPayload},
    error::RequestPreconditionError,
};

/// Represents the request body once unmangled and interpolated, ready to
/// be attached into a request when it's about to be submitted. Request
/// bodies include the raw contents that are added as body, as well as
/// extra headers. These headers are *prepended* to the request, so that
/// if the request headers overload them (say, there's also a "Content-Type"
/// header in the request headers), they will replace whatever the bound
/// request body sets. (This is useful in case you want to send a JSON request
/// but still need to use a custom media type such as application/ld+json.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct BoundRequestBody {
    /// The payload associated with the request body.
    pub content: Option<Vec<u8>>,

    /// The collection of headers to be prepended to the request.
    pub headers: Vec<(String, String)>,
}

impl BoundRequestBody {
    fn try_from_urlencoded(value: &EndpointData) -> Result<Self, RequestPreconditionError> {
        let RequestPayload::Urlencoded(ref kv) = value.body else {
            panic!("Bamboozled by the match");
        };

        let context = value.template_processor();
        let interpolated = kv.render(&context)?;
        let pairs = interpolated.to_active_pairs();
        let body = serde_urlencoded::to_string(pairs)
            .map_err(|_| RequestPreconditionError::EncodingError)?;
        let raw = Vec::from(body.as_str());
        Ok(BoundRequestBody {
            content: Some(raw),
            headers: vec![(
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into(),
            )],
        })
    }

    fn try_from_multipart(value: &EndpointData) -> Result<Self, RequestPreconditionError> {
        let RequestPayload::Multipart { ref params } = value.body else {
            panic!("Bamboozled by the match");
        };

        let context = value.template_processor();
        let interpolated = params.render(&context)?;
        let pairs = interpolated.to_active_pairs();

        let boundary = formdata::generate_boundary();
        let formdata = formdata::FormData {
            fields: pairs,
            files: vec![],
        };
        let mut stream = BufWriter::new(Vec::new());
        formdata::write_formdata(&mut stream, &boundary, &formdata).map_err(|e| {
            glib::g_error!("cartero", "form data error: {}", e);
            RequestPreconditionError::EncodingError
        })?;
        stream
            .flush()
            .map_err(|_| RequestPreconditionError::EncodingError)?;
        let body = stream.get_ref().clone();
        Ok(BoundRequestBody {
            content: Some(body),
            headers: vec![(
                "Content-Type".into(),
                format!(
                    "multipart/form-data; boundary={}",
                    String::from_utf8_lossy(&boundary)
                ),
            )],
        })
    }

    fn try_from_raw(value: &EndpointData) -> Result<Self, RequestPreconditionError> {
        let RequestPayload::Raw {
            ref content,
            ref encoding,
        } = value.body
        else {
            panic!("Bamboozled by the match");
        };
        let processor = value.template_processor();
        let body = String::from_utf8_lossy(content);
        let interpolated = processor.render(&body)?;

        let content_type = match encoding {
            RawEncoding::OctetStream => "application/octet-stream",
            RawEncoding::Xml => "application/xml",
            RawEncoding::Json => "application/json",
        };
        Ok(BoundRequestBody {
            content: Some(Vec::from(interpolated.as_str())),
            headers: vec![("Content-Type".into(), content_type.into())],
        })
    }
}

impl TryFrom<&EndpointData> for BoundRequestBody {
    type Error = RequestPreconditionError;

    fn try_from(value: &EndpointData) -> Result<Self, Self::Error> {
        match value.body {
            RequestPayload::None => Ok(Self {
                content: None,
                headers: vec![],
            }),
            RequestPayload::Urlencoded(..) => Self::try_from_urlencoded(value),
            RequestPayload::Multipart { .. } => Self::try_from_multipart(value),
            RequestPayload::Raw { .. } => Self::try_from_raw(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::KeyValueTable;

    use super::*;

    #[test]
    pub fn test_from_urlencoded() {
        let variables = vec![("API_KEY", "12341234").into()];
        let body = vec![
            ("account_id", "2000").into(),
            ("api_key", "demo.{{API_KEY}}").into(),
        ];
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::new(&variables),
            body: RequestPayload::Urlencoded(KeyValueTable::new(&body)),
        };

        let serial = BoundRequestBody::try_from(&endpoint).unwrap();
        let body = serial.content.unwrap();
        assert_eq!(
            String::from_utf8_lossy(&body),
            "account_id=2000&api_key=demo.12341234"
        );
        assert_eq!(
            serial.headers,
            vec![(
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into()
            ),]
        );
    }

    #[test]
    pub fn test_from_urlencoded_missing_var() {
        let body = vec![
            ("account_id", "2000").into(),
            ("api_key", "demo.{{API_KEY}}").into(),
        ];
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body: RequestPayload::Urlencoded(KeyValueTable::new(&body)),
        };
        let result = BoundRequestBody::try_from(&endpoint);
        assert_eq!(
            result,
            Err(RequestPreconditionError::VariableNotFound(
                "API_KEY".to_string()
            ))
        );
    }

    #[test]
    pub fn test_from_formdata() {
        let variables = vec![("API_KEY", "12341234").into()];
        let body = vec![
            ("account_id", "2000").into(),
            ("api_key", "demo.{{API_KEY}}").into(),
        ];
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::new(&variables),
            body: RequestPayload::Multipart {
                params: KeyValueTable::new(&body),
            },
        };

        let serial = BoundRequestBody::try_from(&endpoint).unwrap();
        let body = serial.content.unwrap();

        // I cannot actually test the boundaries because they are randomly generated...
        let expected_1 = "Content-Type: text/plain\r\nContent-Disposition: form-data; name=\"account_id\"\r\n\r\n2000";
        let expected_2 = "Content-Type: text/plain\r\nContent-Disposition: form-data; name=\"api_key\"\r\n\r\ndemo.12341234";

        let encoded_body = String::from_utf8_lossy(&body);
        assert!(
            encoded_body.contains(expected_1),
            "{} part of {}",
            expected_1,
            encoded_body
        );
        assert!(
            encoded_body.contains(expected_2),
            "{} part of {}",
            expected_2,
            encoded_body
        );

        // Same goes for the header.
        assert_eq!(1, serial.headers.len());
        let (name, value) = &serial.headers[0];
        assert_eq!("Content-Type", name);
        let expected = "multipart/form-data; boundary=";
        assert!(value.contains(expected), "{} contains {}", value, expected);
    }

    #[test]
    pub fn test_from_formdata_missing_variables() {
        let body = vec![
            ("account_id", "2000").into(),
            ("api_key", "demo.{{API_KEY}}").into(),
        ];
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body: RequestPayload::Multipart {
                params: KeyValueTable::new(&body),
            },
        };

        let result = BoundRequestBody::try_from(&endpoint);
        assert_eq!(
            result,
            Err(RequestPreconditionError::VariableNotFound(
                "API_KEY".to_string()
            ))
        );
    }

    #[test]
    pub fn test_from_raw_octet() {
        let body = RequestPayload::Raw {
            encoding: RawEncoding::OctetStream,
            content: Vec::from("hello {{TARGET}}"),
        };
        let variables = vec![("TARGET", "world").into()];
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::new(&variables),
            body,
        };

        let result = BoundRequestBody::try_from(&endpoint).unwrap();
        assert_eq!(
            vec![("Content-Type".into(), "application/octet-stream".into())],
            result.headers
        );
        assert_eq!(Some(Vec::from("hello world")), result.content);
    }

    #[test]
    pub fn test_from_raw_octet_missing_variable() {
        let body = RequestPayload::Raw {
            encoding: RawEncoding::OctetStream,
            content: Vec::from("hello {{TARGET}}"),
        };
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body,
        };

        let result = BoundRequestBody::try_from(&endpoint);
        assert_eq!(
            Err(RequestPreconditionError::VariableNotFound("TARGET".into())),
            result
        );
    }

    #[test]
    pub fn test_from_json() {
        let body = RequestPayload::Raw {
            encoding: RawEncoding::Json,
            content: Vec::from("{\"message\": \"hello {{TARGET}}\"}"),
        };
        let variables = vec![("TARGET", "world").into()];
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::new(&variables),
            body,
        };

        let result = BoundRequestBody::try_from(&endpoint).unwrap();
        assert_eq!(
            vec![("Content-Type".into(), "application/json".into())],
            result.headers
        );
        assert_eq!(
            Some(Vec::from("{\"message\": \"hello world\"}")),
            result.content
        );
    }

    #[test]
    pub fn test_from_json_missing_variable() {
        let body = RequestPayload::Raw {
            encoding: RawEncoding::Json,
            content: Vec::from("{\"message\": \"hello {{TARGET}}\"}"),
        };
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body,
        };

        let result = BoundRequestBody::try_from(&endpoint);
        assert_eq!(
            Err(RequestPreconditionError::VariableNotFound("TARGET".into())),
            result
        );
    }

    #[test]
    pub fn test_from_xml() {
        let body = RequestPayload::Raw {
            encoding: RawEncoding::Xml,
            content: Vec::from("<hello target=\"{{TARGET}}\" />"),
        };
        let variables = vec![("TARGET", "world").into()];
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::new(&variables),
            body,
        };

        let result = BoundRequestBody::try_from(&endpoint).unwrap();
        assert_eq!(
            vec![("Content-Type".into(), "application/xml".into())],
            result.headers
        );
        assert_eq!(
            Some(Vec::from("<hello target=\"world\" />")),
            result.content
        );
    }

    #[test]
    pub fn test_from_xml_missing_variable() {
        let body = RequestPayload::Raw {
            encoding: RawEncoding::Xml,
            content: Vec::from("<hello target=\"{{TARGET}}\" />"),
        };
        let endpoint = EndpointData {
            url: "https://www.example.com/payload".into(),
            method: crate::entities::RequestMethod::Post,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body,
        };

        let result = BoundRequestBody::try_from(&endpoint);
        assert_eq!(
            Err(RequestPreconditionError::VariableNotFound("TARGET".into())),
            result
        );
    }
}
