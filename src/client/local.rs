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

use std::collections::HashMap;
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
    if !url.contains("://") {
        return Err(RequestPreconditionError::MissingProtocol);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::*;
    use crate::error::RequestPreconditionError;

    #[test]
    pub fn test_url_normalization() {
        let positive = vec![
            // These are straightforward.
            "https://example.com/api/users",
            "http://example.com/api/users",
            "http://example.com:8080/api/users",
            "https://example.com:8043/api/users",
            // IP address
            "https://192.168.1.4/api/users",
            "http://192.168.1.4/api/users",
            "http://192.168.1.4",
            "https://192.168.1.4",
            "http://192.168.1.4:4000",
            "https://192.168.1.4:4000/api/users",
            // These should work too.
            "https://localhost:3000/api/users",
            "http://localhost:3000/api/users",
            "http://localhost:3000",
            "http://localhost",
            "http://localhost/api/users",
            // Basic authentication in the URL
            "http://admin:admin@router/login",
            "http://admin:admin@router.home/login",
            "http://admin:admin@192.168.1.1/login",
        ];

        let wrong_protocol = vec!["gemini://geminiprotocol.net", "ws://localhost:8000/socket"];

        let missing_protocol = vec![
            // Typical use in development
            "example.com/api/users",
            "example.com",
            "example.com:8000",
            "192.168.1.4",
            "192.168.1.4/api/users",
            "192.168.1.4:8000",
            "localhost",
            "localhost/users",
            "localhost:3000/users",
            // This is open to discussion but for this use case, it's an HTTP request to example.com
            // using user=mailto and pass=foobar as a basic authentication, and not an email request.
            "mailto:foobar@example.com",
            "mailto:foobar@example.com/api/users",
        ];

        for url in positive {
            let result = super::normalize_url(url);
            assert!(
                result.is_ok(),
                "{} should have been accepted, was {:?}",
                url,
                result
            );
        }

        for url in wrong_protocol {
            let result = super::normalize_url(url);
            let Err(RequestPreconditionError::UnsupportedProtocol(_)) = result else {
                panic!("{} should have been an unsupported protocol", url);
            };
        }

        for url in missing_protocol {
            let result = super::normalize_url(url);
            assert_eq!(Err(RequestPreconditionError::MissingProtocol), result);
        }
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
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
            parameters: KeyValueTable::default(),
        };
        let result = BoundRequest::try_from(endpoint).unwrap();
        assert_eq!(result.url, "https://example.com/api/v1/users");
    }
}
