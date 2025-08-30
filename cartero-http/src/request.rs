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

use cartero_objects::{FieldTable, Request, RequestMethod};

use crate::{
    active_pairs, auth::BoundHeaders, body::BoundBody, url::normalize_url, RequestEnvironment,
    RequestError,
};

pub struct BoundRequest {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl BoundRequest {
    pub async fn new(value: &Request, env: &RequestEnvironment) -> Result<Self, RequestError> {
        if value.url().trim().is_empty() {
            return Err(RequestError::EmptyUrl);
        }

        let resolved = value.resolve()?;
        let url = normalize_url(&resolved.url())?;

        let method = resolved.method();

        let user_headers = resolved.headers();
        let auth = BoundHeaders::try_from(value)?;
        let body = BoundBody::new(value, env).await?;
        let headers = combine_headers(&user_headers, &auth, &body);

        Ok(Self {
            url,
            method,
            headers,
            body: body.body(),
        })
    }
}

fn combine_headers(
    headers: &FieldTable,
    auth: &BoundHeaders,
    body: &BoundBody,
) -> HashMap<String, String> {
    // Priority: auth + body < headers. So first we process auth and body, and
    // then maybe override the pregenerated headers with whatever the user has
    // typed in the Headers tab.

    let mut combined = HashMap::new();
    combined.extend(auth.headers());
    combined.extend(body.headers());

    let pairs = active_pairs(headers);
    combined.extend(pairs);
    combined
}

#[cfg(test)]
mod tests {
    use cartero_objects::{
        Field, Request, RequestAuthentication, RequestAuthenticationBasic, RequestBody,
        RequestBodyUrlencoded, RequestMethod,
    };

    use crate::{BoundRequest, RequestEnvironment, RequestError};

    fn dummy_env() -> RequestEnvironment {
        RequestEnvironment {
            prefix: None,
            proxy: None,
            config: crate::ClientConfig {
                validate_tls: false,
                redirects: 0,
                timeout: 30.0,
            },
        }
    }

    #[tokio::test]
    async fn test_convert() {
        let req = Request::builder(
            "https://www.example.com/api/v1/users",
            cartero_objects::RequestMethod::Get,
        )
        .build();

        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(bound.method, RequestMethod::Get);
        assert_eq!(0, bound.headers.len());
        assert!(bound.body.is_none());
    }

    #[tokio::test]
    async fn test_convert_empty_url() {
        let env = dummy_env();
        let req = Request::builder("    ", cartero_objects::RequestMethod::Get).build();
        match BoundRequest::new(&req, &env).await {
            Err(RequestError::EmptyUrl) => {}
            Err(other) => panic!("Failed with an unknown condition: {:?}", other),
            _ => panic!("Expected a failure"),
        };
    }

    #[tokio::test]
    async fn test_convert_missing_protocol() {
        let env = dummy_env();
        let req = Request::builder("localhost:3000", cartero_objects::RequestMethod::Get).build();
        match BoundRequest::new(&req, &env).await {
            Err(RequestError::MissingProtocol) => {}
            Err(other) => panic!("Failed with an unknown condition: {:?}", other),
            _ => panic!("Expected a failure"),
        };
    }

    #[tokio::test]
    async fn test_convert_unsupported_protocol() {
        let req = Request::builder(
            "ftp://ftp.gnu.org/gnu/hello/hello-2.12.tar.gz",
            cartero_objects::RequestMethod::Get,
        )
        .build();
        let env = dummy_env();
        match BoundRequest::new(&req, &env).await {
            Err(RequestError::UnsupportedProtocol(proto)) => assert_eq!(proto, "ftp"),
            Err(other) => panic!("Failed with an unknown condition: {:?}", other),
            _ => panic!("Expected a failure"),
        };
    }

    #[tokio::test]
    async fn test_convert_variable_in_url() {
        let req = Request::builder(
            "{{ API_ROOT }}/api/v1/users",
            cartero_objects::RequestMethod::Get,
        )
        .variable(
            &Field::builder()
                .key("API_ROOT")
                .value("https://www.example.com")
                .build(),
        )
        .build();

        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(bound.method, RequestMethod::Get);
        assert_eq!(0, bound.headers.len());
        assert!(bound.body.is_none());
    }

    #[tokio::test]
    async fn test_convert_variable_in_url_without_variable() {
        let req = Request::builder(
            "{{ API_ROOT }}/api/v1/users",
            cartero_objects::RequestMethod::Get,
        )
        .build();

        let env = dummy_env();
        match BoundRequest::new(&req, &env).await {
            Err(RequestError::VariableNotFound(var)) => assert_eq!(var, "API_ROOT"),
            Err(other) => panic!("Failed with an unknown condition: {:?}", other),
            _ => panic!("Expected a failure"),
        };
    }

    #[tokio::test]
    async fn test_auth_is_added_to_headers() {
        let auth = RequestAuthenticationBasic::builder()
            .username("admin")
            .password("admin")
            .build();
        let auth = RequestAuthentication::builder().basic_auth(&auth).build();
        let req = Request::builder(
            "https://www.example.com/api/v1/users",
            cartero_objects::RequestMethod::Get,
        )
        .with_auth(auth)
        .build();
        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(RequestMethod::Get, bound.method);
        assert!(bound.body.is_none());
        assert_eq!(bound.headers["Authorization"], "Basic YWRtaW46YWRtaW4=");
    }

    #[tokio::test]
    async fn test_headers_can_override_auth() {
        // It's dumb, but yeah, you totally can.
        let auth = RequestAuthenticationBasic::builder()
            .username("admin")
            .password("admin")
            .build();
        let auth = RequestAuthentication::builder().basic_auth(&auth).build();
        let req = Request::builder(
            "https://www.example.com/api/v1/users",
            cartero_objects::RequestMethod::Get,
        )
        .with_auth(auth)
        .header(
            &Field::builder()
                .key("Authorization")
                .value("Bearer 1234")
                .build(),
        )
        .build();
        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(RequestMethod::Get, bound.method);
        assert!(bound.body.is_none());
        assert_eq!(bound.headers["Authorization"], "Bearer 1234");
    }

    #[tokio::test]
    async fn test_headers_cannot_override_auth_if_disabled() {
        let auth = RequestAuthenticationBasic::builder()
            .username("admin")
            .password("admin")
            .build();
        let auth = RequestAuthentication::builder().basic_auth(&auth).build();
        let req = Request::builder(
            "https://www.example.com/api/v1/users",
            cartero_objects::RequestMethod::Get,
        )
        .with_auth(auth)
        .header(
            &Field::builder()
                .key("Authorization")
                .value("Bearer 1234")
                .active(false)
                .build(),
        )
        .build();
        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(RequestMethod::Get, bound.method);
        assert!(bound.body.is_none());
        assert_eq!(bound.headers["Authorization"], "Basic YWRtaW46YWRtaW4=");
    }

    #[tokio::test]
    async fn test_body_is_added_to_headers() {
        let body = RequestBodyUrlencoded::builder()
            .field(&Field::builder().key("user_id").value("10").build())
            .field(&Field::builder().key("category_id").value("2").build())
            .build();
        let body = RequestBody::builder().urlencoded(&body).build();
        let req = Request::builder("https://www.example.com/api/v1/users", RequestMethod::Post)
            .with_body(body)
            .build();
        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(RequestMethod::Post, bound.method);
        assert_eq!(
            bound.headers["Content-Type"],
            "application/x-www-form-urlencoded"
        );
        let body = bound.body.unwrap();
        assert_eq!(b"user_id=10&category_id=2", body.as_slice());
    }

    #[tokio::test]
    async fn test_headers_can_override_body() {
        let body = RequestBodyUrlencoded::builder()
            .field(&Field::builder().key("user_id").value("10").build())
            .field(&Field::builder().key("category_id").value("2").build())
            .build();
        let body = RequestBody::builder().urlencoded(&body).build();
        let req = Request::builder("https://www.example.com/api/v1/users", RequestMethod::Post)
            .with_body(body)
            .header(
                &Field::builder()
                    .key("Content-Type")
                    .value("application/octet-stream")
                    .build(),
            )
            .build();
        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(RequestMethod::Post, bound.method);
        assert_eq!(bound.headers["Content-Type"], "application/octet-stream");
        let body = bound.body.unwrap();
        assert_eq!(b"user_id=10&category_id=2", body.as_slice());
    }

    #[tokio::test]
    async fn test_headers_cannot_override_body_if_disabled() {
        let body = RequestBodyUrlencoded::builder()
            .field(&Field::builder().key("user_id").value("10").build())
            .field(&Field::builder().key("category_id").value("2").build())
            .build();
        let body = RequestBody::builder().urlencoded(&body).build();
        let req = Request::builder("https://www.example.com/api/v1/users", RequestMethod::Post)
            .with_body(body)
            .header(
                &Field::builder()
                    .key("Content-Type")
                    .value("application/octet-stream")
                    .active(false)
                    .build(),
            )
            .build();
        let env = dummy_env();
        let bound = BoundRequest::new(&req, &env).await.unwrap();
        assert_eq!(bound.url, "https://www.example.com/api/v1/users");
        assert_eq!(RequestMethod::Post, bound.method);
        assert_eq!(
            bound.headers["Content-Type"],
            "application/x-www-form-urlencoded"
        );
        let body = bound.body.unwrap();
        assert_eq!(b"user_id=10&category_id=2", body.as_slice());
    }
}
