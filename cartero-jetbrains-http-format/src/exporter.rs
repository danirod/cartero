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

use cartero_http::{ClientConfig, RequestEnvironment, RequestError};
use cartero_objects::Request;

pub(crate) async fn export(request: &Request) -> Result<String, RequestError> {
    let env = RequestEnvironment {
        config: ClientConfig {
            redirects: 0,
            timeout: 30.0,
            validate_tls: false,
        },
        prefix: None,
    };

    let request = cartero_http::BoundRequest::new(&request, &env).await?;

    let verb = request.method.to_string();
    let url = request.url;
    let headers = request
        .headers
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<String>>()
        .join("\n");
    let body = request
        .body
        .map(|body| {
            let string_body = String::from_utf8_lossy(&body);
            format!("\n\n{string_body}")
        })
        .unwrap_or_default();

    let output = vec![format!("{} {}", verb, url), headers, body]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join("\n");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use cartero_objects::{
        Field, RequestAuthentication, RequestAuthenticationBasic, RequestAuthenticationBearer,
        RequestBody, RequestBodyFile, RequestBodyMultipart, RequestBodyRaw, RequestBodyRawType,
        RequestBodyUrlencoded, RequestMethod,
    };

    use super::*;

    #[tokio::test]
    async fn test_converts_simple_request() {
        let request =
            Request::builder("https://www.example.com/index.html", RequestMethod::Post).build();
        let result = export(&request).await.expect("Must succeed");
        assert_eq!("POST https://www.example.com/index.html", result);
    }

    #[tokio::test]
    async fn test_resolves_variables_in_url() {
        let request = Request::builder("{{ API_ROOT }}/index.html", RequestMethod::Post)
            .variable(
                &Field::builder()
                    .key("API_ROOT")
                    .value("https://www.example.com")
                    .build(),
            )
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert_eq!("POST https://www.example.com/index.html", result);
    }

    #[tokio::test]
    async fn test_converts_headers() {
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .header(
                &Field::builder()
                    .key("Accept")
                    .value("application/xml")
                    .build(),
            )
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Accept: application/xml"));
    }

    #[tokio::test]
    async fn test_skips_disabled_headers() {
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .header(
                &Field::builder()
                    .key("Accept")
                    .value("application/xml")
                    .active(false)
                    .build(),
            )
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(!result.contains("Accept: application/xml"));
    }

    #[tokio::test]
    async fn test_resolves_variables_in_headers() {
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .variable(&Field::builder().key("API_KEY").value("12341234").build())
            .header(
                &Field::builder()
                    .key("X-Api-Key")
                    .value("{{ API_KEY }}")
                    .build(),
            )
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("X-Api-Key: 12341234"));
    }

    #[tokio::test]
    async fn test_resolves_basic_auth() {
        let basic_auth = RequestAuthenticationBasic::builder()
            .username("admin")
            .password("1234")
            .build();
        let auth = RequestAuthentication::builder()
            .basic_auth(&basic_auth)
            .build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_auth(auth)
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Authorization: Basic YWRtaW46MTIzNA=="));
    }

    #[tokio::test]
    async fn test_resolves_bearer() {
        let bearer_token = RequestAuthenticationBearer::builder()
            .token("my_little_pony")
            .build();
        let auth = RequestAuthentication::builder()
            .bearer_token(&bearer_token)
            .build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_auth(auth)
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Authorization: Bearer my_little_pony"));
    }

    #[tokio::test]
    async fn test_resolves_urlencoded_bodies() {
        let urlencoded = RequestBodyUrlencoded::builder()
            .field(&Field::builder().key("category").value("10").build())
            .field(
                &Field::builder()
                    .key("user_id")
                    .value("1")
                    .active(false)
                    .build(),
            )
            .field(&Field::builder().key("user_id").value("20").build())
            .field(&Field::builder().key("user_id").value("25").build())
            .field(&Field::builder().key("label").value("send value").build())
            .build();
        let body = RequestBody::builder().urlencoded(&urlencoded).build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_body(body)
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Content-Type: application/x-www-form-urlencoded"));
        assert!(result.contains("\n\ncategory=10&user_id=20&user_id=25&label=send+value"));
    }

    #[tokio::test]
    async fn test_resolves_multipart_bodies() {
        let multipart = RequestBodyMultipart::builder()
            .field(&Field::builder().key("category").value("10").build())
            .field(
                &Field::builder()
                    .key("user_id")
                    .value("1")
                    .active(false)
                    .build(),
            )
            .field(&Field::builder().key("user_id").value("20").build())
            .field(&Field::builder().key("user_id").value("25").build())
            .field(&Field::builder().key("label").value("send value").build())
            .build();
        let body = RequestBody::builder().multipart(&multipart).build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_body(body)
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Content-Type: multipart/form-data; boundary="));
        assert!(result.contains("Content-Disposition: form-data; name=\"category\""));
        assert!(result.contains("\r\n\r\n10\r\n"));
        assert!(result.contains("Content-Disposition: form-data; name=\"user_id\""));
        assert!(result.contains("\r\n\r\n20\r\n"));
        assert!(result.contains("\r\n\r\n25\r\n"));
        assert!(result.contains("Content-Disposition: form-data; name=\"label\""));
        assert!(result.contains("\r\n\r\nsend value\r\n"));
    }

    #[tokio::test]
    async fn test_resolves_octet_stream_bodies() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
            .payload("hello world")
            .build();
        let body = RequestBody::builder().raw(&raw).build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_body(body)
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Content-Type: application/octet-stream"));
        assert!(result.contains("\n\nhello world"));
    }

    #[tokio::test]
    async fn test_resolves_json_bodies() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::Json)
            .payload("{\"hello\": \"world\"}")
            .build();
        let body = RequestBody::builder().raw(&raw).build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_body(body)
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Content-Type: application/json"));
        assert!(result.contains("\n\n{\"hello\": \"world\"}"));
    }

    #[tokio::test]
    async fn test_resolves_xml_bodies() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::Xml)
            .payload("<?xml version=\"1.0\" ?><head />")
            .build();
        let body = RequestBody::builder().raw(&raw).build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_body(body)
            .build();
        let result = export(&request).await.expect("Must succeed");
        assert!(result.contains("Content-Type: application/xml"));
        assert!(result.contains("\n\n<?xml version=\"1.0\" ?><head />"));
    }

    #[tokio::test]
    async fn test_resolves_file_bodies() {
        let file = RequestBodyFile::builder().path("response.json").build();
        let body = RequestBody::builder().file(&file).build();
        let request = Request::builder("https://www.example.com/index.html", RequestMethod::Post)
            .with_body(body)
            .build();
        let result = export(&request).await;
        assert!(matches!(result, Err(RequestError::FilePrefixUnset)));
    }
}
