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

use formdata::FormData;
use isahc::http::header::{InvalidHeaderName, InvalidHeaderValue};
use srtemplate::SrTemplate;
use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};
use thiserror::Error;

use crate::{
    entities::{EndpointData, KeyValueTable, RawEncoding, RequestMethod, RequestPayload},
    error::CarteroError,
};

#[derive(Default, Debug, Clone)]
pub struct BoundRequest {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Default, Debug, Clone)]
struct BoundBody {
    content: Vec<u8>,
    boundary: String,
}

fn bind_urlencoded_payload(
    body: &KeyValueTable,
    processor: &SrTemplate,
) -> Result<Option<BoundBody>, CarteroError> {
    if body.is_empty() {
        return Ok(None);
    }
    let pairs: Vec<(String, String)> = body
        .iter()
        .filter(|var| var.active)
        .map(|var| {
            let key = processor.render(var.name.clone())?;
            let value = processor.render(var.value.clone())?;
            Ok((key, value))
        })
        .collect::<Result<Vec<(String, String)>, CarteroError>>()?;
    let body = serde_urlencoded::to_string(pairs).map_err(|_| RequestError::InvalidPayload)?;
    let content = Vec::from(body.as_str());
    Ok(Some(BoundBody {
        content,
        boundary: String::default(),
    }))
}

fn bind_multipart_payload(
    params: &KeyValueTable,
    processor: &SrTemplate,
) -> Result<Option<BoundBody>, CarteroError> {
    if params.is_empty() {
        return Ok(None);
    }
    let pairs: Vec<(String, String)> = params
        .iter()
        .filter(|var| var.active)
        .map(|var| {
            let key = processor.render(var.name.clone())?;
            let value = processor.render(var.value.clone())?;
            Ok((key, value))
        })
        .collect::<Result<Vec<(String, String)>, CarteroError>>()?;
    let formdata = FormData {
        fields: pairs,
        files: vec![],
    };
    let boundary = formdata::generate_boundary();

    let mut stream = BufWriter::new(Vec::new());
    formdata::write_formdata(&mut stream, &boundary, &formdata)
        .map_err(|_| RequestError::InvalidPayload)?;
    stream.flush().map_err(RequestError::IOError)?;
    let copy = stream.get_ref().clone();
    Ok(Some(BoundBody {
        content: copy,
        boundary: String::from_utf8_lossy(&boundary).to_string(),
    }))
}

fn bind_raw_payload(
    body: &[u8],
    processor: &SrTemplate,
) -> Result<Option<BoundBody>, CarteroError> {
    if body.is_empty() {
        return Ok(None);
    }
    let processable_body = String::from_utf8_lossy(body);
    let processed_body = processor.render(processable_body)?;
    let body = Vec::from(processed_body.as_str());
    Ok(Some(BoundBody {
        content: body,
        boundary: String::default(),
    }))
}

fn bind_payload(
    body: &RequestPayload,
    processor: &SrTemplate,
) -> Result<Option<BoundBody>, CarteroError> {
    match body {
        RequestPayload::None => Ok(None),
        RequestPayload::Urlencoded(payload) => bind_urlencoded_payload(payload, processor),
        RequestPayload::Multipart { params } => bind_multipart_payload(params, processor),
        RequestPayload::Raw {
            content,
            encoding: _,
        } => bind_raw_payload(content, processor),
    }
}

impl TryFrom<EndpointData> for BoundRequest {
    type Error = CarteroError;

    fn try_from(value: EndpointData) -> Result<Self, Self::Error> {
        let processor = value.template_processor();

        let url = processor.render(&value.url)?;
        let method = value.method.clone();

        let body = bind_payload(&value.body, &processor)?;
        let content_type = match value.body {
            RequestPayload::None => None,
            RequestPayload::Urlencoded(_) => Some("application/x-www-form-urlencoded".to_string()),
            RequestPayload::Multipart { params: _ } => Some(format!(
                "multipart/form-data; boundary={}",
                body.clone().unwrap_or_default().boundary
            )),
            RequestPayload::Raw {
                ref encoding,
                content: _,
            } => match encoding {
                RawEncoding::OctetStream => Some("application/octet-stream".into()),
                RawEncoding::Xml => Some("application/xml".into()),
                RawEncoding::Json => Some("application/json".into()),
            },
        };

        let mut base_headers = HashMap::new();
        if let Some(content_type) = content_type {
            base_headers.insert("Content-Type".to_string(), content_type);
        }
        base_headers.extend(value.process_headers());

        let headers: Result<HashMap<String, String>, CarteroError> = base_headers
            .iter()
            .map(|(k, v)| {
                let header_name = processor.render(k)?;
                let header_value = processor.render(v)?;
                Ok((header_name, header_value))
            })
            .collect();
        let headers = headers?;

        Ok(Self {
            url,
            method,
            headers,
            body: body.map(|b| b.content),
        })
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Illegal HTTP verb")]
    InvalidHttpVerb,

    #[error("Invalid headers state")]
    InvalidHeaders,

    #[error("Invalid payload state")]
    InvalidPayload,

    #[error("Illegal header")]
    InvalidHeaderName(#[from] InvalidHeaderName),

    #[error("Illegal header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("Request error")]
    NetworkError(#[from] isahc::error::Error),

    #[error("HTTP error")]
    HttpError(#[from] isahc::http::Error),

    #[error("Unknown I/O error")]
    IOError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use crate::entities::KeyValueTable;

    use super::*;

    #[test]
    pub fn test_bind_of_parameters_urlencoded() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Post;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_ID", "123412341234").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = RequestPayload::Urlencoded(KeyValueTable::new(&[
            ("name", "John").into(),
            ("surname", "Smith").into(),
        ]));
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let bound = BoundRequest::try_from(endpoint).unwrap();

        assert_eq!(bound.url, "https://api.example.com/v1/books");
        assert_eq!(bound.headers["Authorization"], "Bearer 789078907890");
        assert_eq!(
            bound.headers["Content-Type"],
            "application/x-www-form-urlencoded"
        );
        assert_eq!(bound.body, Some(Vec::from(b"name=John&surname=Smith")));
    }

    #[test]
    pub fn test_bind_of_parameters_formdata() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Post;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_ID", "123412341234").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = RequestPayload::Multipart {
            params: KeyValueTable::new(&[("name", "John").into(), ("surname", "Smith").into()]),
        };
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let bound = BoundRequest::try_from(endpoint).unwrap();

        assert_eq!(bound.url, "https://api.example.com/v1/books");
        assert_eq!(bound.headers["Authorization"], "Bearer 789078907890");

        let content_type = bound.headers["Content-Type"].clone();
        assert!(content_type.starts_with("multipart/form-data; boundary="));
        let body = bound.body.unwrap();
        let body = String::from_utf8_lossy(&body);
        assert!(body.contains("name=\"name\""));
        assert!(body.contains("name=\"surname\""));
        assert!(body.contains("John"));
        assert!(body.contains("Smith"));
    }

    #[test]
    pub fn test_bind_of_parameters_json() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Post;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_ID", "123412341234").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = RequestPayload::Raw {
            encoding: RawEncoding::Json,
            content: Vec::from(b"{\"hello\": \"world\"}"),
        };
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let bound = BoundRequest::try_from(endpoint).unwrap();

        assert_eq!(bound.url, "https://api.example.com/v1/books");
        assert_eq!(bound.headers["Authorization"], "Bearer 789078907890");
        assert_eq!(bound.headers["Content-Type"], "application/json");

        let body = bound.body.unwrap();
        let body = String::from_utf8_lossy(&body);
        assert_eq!(body, "{\"hello\": \"world\"}");
    }

    #[test]
    pub fn test_bind_of_parameters_xml() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Post;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_ID", "123412341234").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = RequestPayload::Raw {
            encoding: RawEncoding::Xml,
            content: Vec::from(b"<envelope>1234</envelope>"),
        };
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let bound = BoundRequest::try_from(endpoint).unwrap();

        assert_eq!(bound.url, "https://api.example.com/v1/books");
        assert_eq!(bound.headers["Authorization"], "Bearer 789078907890");
        assert_eq!(bound.headers["Content-Type"], "application/xml");

        let body = bound.body.unwrap();
        let body = String::from_utf8_lossy(&body);
        assert_eq!(body, "<envelope>1234</envelope>");
    }

    #[test]
    pub fn test_bind_of_parameters_octet() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Post;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_ID", "123412341234").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = RequestPayload::Raw {
            encoding: RawEncoding::OctetStream,
            content: Vec::from(b"12341234"),
        };
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let bound = BoundRequest::try_from(endpoint).unwrap();

        assert_eq!(bound.url, "https://api.example.com/v1/books");
        assert_eq!(bound.headers["Authorization"], "Bearer 789078907890");
        assert_eq!(bound.headers["Content-Type"], "application/octet-stream");

        let body = bound.body.unwrap();
        let body = String::from_utf8_lossy(&body);
        assert_eq!(body, "12341234");
    }

    #[test]
    pub fn test_bind_of_parameters_may_still_override_header() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Post;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
            ("Content-Type", "application/ld+json").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_ID", "123412341234").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = RequestPayload::Raw {
            encoding: RawEncoding::Json,
            content: Vec::from(b"{\"hello\": \"world\"}"),
        };
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let bound = BoundRequest::try_from(endpoint).unwrap();

        assert_eq!(bound.url, "https://api.example.com/v1/books");
        assert_eq!(bound.headers["Authorization"], "Bearer 789078907890");
        assert_eq!(bound.headers["Content-Type"], "application/ld+json");

        let body = bound.body.unwrap();
        let body = String::from_utf8_lossy(&body);
        assert_eq!(body, "{\"hello\": \"world\"}");
    }

    #[test]
    fn test_bind_with_duplicate_headers() {
        let url = "https://www.example.com/v1/books".into();
        let method = RequestMethod::Get;
        let headers = KeyValueTable::new(&[
            ("Accept", "application/html").into(),
            ("Accept", "application/xml").into(),
        ]);
        let variables = KeyValueTable::default();
        let body = RequestPayload::None;
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        let bound = BoundRequest::try_from(endpoint).unwrap();
        assert_eq!(bound.headers["Accept"], "application/xml")
    }

    #[test]
    fn test_bind_with_duplicate_variables() {
        let url = "https://www.example.com/v1/books".into();
        let method = RequestMethod::Get;
        let headers = KeyValueTable::new(&[("Accept", "{{TYPE}}").into()]);
        let variables = KeyValueTable::new(&[
            ("TYPE", "text/html").into(),
            ("TYPE", "application/json").into(),
        ]);
        let body = RequestPayload::None;
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        let bound = BoundRequest::try_from(endpoint).unwrap();
        assert_eq!(bound.headers["Accept"], "application/json")
    }

    #[test]
    #[should_panic]
    pub fn test_panics_if_wrong_variable() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Get;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = RequestPayload::None;
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let _ = BoundRequest::try_from(endpoint).unwrap();
    }
}
