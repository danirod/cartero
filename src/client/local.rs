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

use isahc::http::header::{InvalidHeaderName, InvalidHeaderValue};
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

use crate::{
    entities::{EndpointData, RequestMethod},
    error::RequestPreconditionError,
};

use super::request_bodies::BoundRequestBody;

#[derive(Default, Debug, Clone)]
pub struct BoundRequest {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

fn normalize_url(url: &str) -> Result<String, RequestPreconditionError> {
    match Url::parse(url) {
        Ok(url) => {
            // Check for protocol as well.
            match url.scheme() {
                "http" | "https" => Ok(url.to_string()),
                other => Err(RequestPreconditionError::UnsupportedProtocol(
                    other.to_owned(),
                )),
            }
        }
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            Err(RequestPreconditionError::MissingProtocol)
        }
        Err(_) => Err(RequestPreconditionError::UrlBadParse),
    }
}

impl TryFrom<EndpointData> for BoundRequest {
    type Error = RequestPreconditionError;

    fn try_from(value: EndpointData) -> Result<Self, Self::Error> {
        let processor = value.template_processor();

        let url = processor.render(&value.url)?;
        let url = normalize_url(&url)?;

        let method = value.method.clone();
        let headers = value.headers.render(&processor)?;

        let body = BoundRequestBody::try_from(&value)?;

        // Use the request body headers to craft the real request headers.
        let headers = {
            let mut all_headers = HashMap::from_iter(body.headers);
            all_headers.extend(headers.to_active_pairs());
            all_headers
        };

        Ok(Self {
            url,
            method,
            headers,
            body: body.content,
        })
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Illegal HTTP verb")]
    InvalidHttpVerb,

    #[error("Invalid URL")]
    InvalidUrl,

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
    use crate::entities::*;

    use super::*;

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

    #[test]
    pub fn test_fails_if_url_lacks_protocol() {
        let url = "example.com/api/v1/users";
        let method = RequestMethod::Get;

        let endpoint = EndpointData {
            url: url.into(),
            method,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body: RequestPayload::None,
        };
        let result = BoundRequest::try_from(endpoint);
        assert!(result.is_err_and(|e| e == RequestPreconditionError::MissingProtocol));
    }

    #[test]
    pub fn test_fails_if_url_uses_unacceptable_protocol() {
        let url = "gemini://geminiprotocol.net/";
        let method = RequestMethod::Get;

        let endpoint = EndpointData {
            url: url.into(),
            method,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body: RequestPayload::None,
        };
        let result = BoundRequest::try_from(endpoint);
        assert!(result.is_err_and(
            |e| e == RequestPreconditionError::UnsupportedProtocol("gemini".to_string())
        ));
    }

    #[test]
    pub fn test_normalizes_url_with_leading_spaces() {
        let url = " https://example.com/api/v1/users";
        let method = RequestMethod::Get;

        let endpoint = EndpointData {
            url: url.into(),
            method,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body: RequestPayload::None,
        };
        let result = BoundRequest::try_from(endpoint).unwrap();
        assert_eq!(result.url, "https://example.com/api/v1/users");
    }

    #[test]
    pub fn test_normalizes_url_with_trailing_spaces() {
        let url = "https://example.com/api/v1/users ";
        let method = RequestMethod::Get;

        let endpoint = EndpointData {
            url: url.into(),
            method,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body: RequestPayload::None,
        };
        let result = BoundRequest::try_from(endpoint).unwrap();
        assert_eq!(result.url, "https://example.com/api/v1/users");
    }

    #[test]
    pub fn test_normalizes_url_with_leading_and_trailing_spaces() {
        let url = " https://example.com/api/v1/users ";
        let method = RequestMethod::Get;

        let endpoint = EndpointData {
            url: url.into(),
            method,
            headers: KeyValueTable::default(),
            variables: KeyValueTable::default(),
            body: RequestPayload::None,
        };
        let result = BoundRequest::try_from(endpoint).unwrap();
        assert_eq!(result.url, "https://example.com/api/v1/users");
    }
}
