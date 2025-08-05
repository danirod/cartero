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

use cartero_objects::{Field, FieldTable, Request, RequestBodyRawType, RequestBodyType};
use gtk::gio::prelude::ListModelExtManual;
use serde_json::{Error, Value};

fn escape_single_string(str: impl AsRef<str>) -> String {
    str.as_ref().replace("'", "'\\''")
}

fn encode_multipart(params: &FieldTable) -> String {
    params
        .iter::<Field>()
        .filter_map(|f| f.ok())
        .filter(|f| f.active())
        .map(|f| {
            format!(
                "-F '{}={}'",
                escape_single_string(f.key()),
                escape_single_string(f.value())
            )
        })
        .collect::<Vec<_>>()
        .join(" \\\n  ")
}

pub fn export_as_curl(request: &Request) -> Result<String, cartero_http::RequestError> {
    let bound_request = cartero_http::BoundRequest::try_from(request.clone())?;
    let mut command = "curl".to_string();
    let template = request.variables().template_processor();

    command.push_str(&{
        let method_str: String = bound_request.method.to_string();
        format!(" -X {} '{}'", method_str, bound_request.url)
    });

    if !bound_request.headers.is_empty() {
        let size = bound_request.headers.len();
        let mut keys: Vec<String> = bound_request.headers.keys().cloned().collect();
        keys.sort();

        // Skip the Content-Type in multipart requests because it is ugly anyway
        // and cURL will add its own boundary anyway.
        if request.body().body_type() == RequestBodyType::Multipart {
            keys.retain(|k| !k.eq("Content-Type"));
        }

        command.push_str(" \\\n");

        for (i, key) in keys.iter().enumerate() {
            let val = bound_request.headers.get(key).unwrap();

            command.push_str(&{
                let mut initial = format!("  -H '{key}: {val}'");

                if i < size - 1 {
                    initial.push_str(" \\\n");
                }

                initial
            });
        }
    }

    if let Some(_) = request.body().urlencoded() {
        if let Some(bd) = bound_request.body.clone() {
            let str = String::from_utf8_lossy(&bd).to_string();
            command.push_str(&format!(" \\\n  -d '{str}'"));
        }
    }

    if let Some(multipart) = request.body().multipart() {
        let table = multipart.params().render(&template)?;
        let params = encode_multipart(&table);
        command.push_str(&format!(" \\\n  {params}"));
    }

    if let Some(raw) = request.body().raw() {
        let content = bound_request.body.clone().unwrap_or_default();
        let encoding = raw.payload_type();
        match encoding {
            RequestBodyRawType::Json => {
                command.push_str(&'fmt: {
                    let body = String::from_utf8_lossy(&content).to_string();
                    let value: Result<Value, Error> = serde_json::from_str(body.as_ref());

                    if value.is_err() {
                        break 'fmt String::new();
                    }

                    let value = value.unwrap();
                    let trimmed_json_str = serde_json::to_string(&value);

                    if trimmed_json_str.is_err() {
                        break 'fmt String::new();
                    }

                    let trimmed_json_str = trimmed_json_str.unwrap();
                    let trimmed_json_str = trimmed_json_str.replace("'", "\\\\'");

                    format!(" \\\n  -d '{}'", trimmed_json_str)
                });
            }
            _ => {
                command.push_str(&{
                    let string = String::from_utf8_lossy(&content).to_string();
                    format!(" \\\n  -d '{}'", string)
                });
            }
        }
    }

    Ok(command)
}

#[cfg(test)]
mod tests {
    use cartero_objects::{
        Field, RequestAuthentication, RequestAuthenticationBasic, RequestAuthenticationBearer,
        RequestBody, RequestBodyMultipart, RequestBodyRaw, RequestBodyUrlencoded, RequestMethod,
    };

    use super::*;

    #[test]
    fn test_export_simple_as_curl() {
        let request =
            Request::builder("https://www.example.com/foobar", RequestMethod::Get).build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("curl -X GET 'https://www.example.com/foobar'"),
            "Expected {} to be valid",
            output
        );
    }

    #[test]
    fn test_export_uses_http_method() {
        let cases = [
            (
                RequestMethod::Get,
                "curl -X GET 'https://www.example.com/foobar'",
            ),
            (
                RequestMethod::Post,
                "curl -X POST 'https://www.example.com/foobar'",
            ),
            (
                RequestMethod::Put,
                "curl -X PUT 'https://www.example.com/foobar'",
            ),
            (
                RequestMethod::Delete,
                "curl -X DELETE 'https://www.example.com/foobar'",
            ),
        ];
        for (verb, expected) in cases {
            let request = Request::builder("https://www.example.com/foobar", verb).build();
            let output = super::export_as_curl(&request).expect("Should have been OK");
            assert!(
                output.contains(expected),
                "Expected {} to contain {}",
                output,
                expected
            );
        }
    }

    #[test]
    fn test_includes_headers() {
        let request = Request::builder("https://www.example.com/foobar", RequestMethod::Get)
            .header(
                &Field::builder()
                    .key("User-Agent")
                    .value("Mozilla/5.0")
                    .build(),
            )
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("-H 'User-Agent: Mozilla/5.0'"),
            "Expected {} to contain the header",
            output
        );
    }

    #[test]
    fn test_renders_variables() {
        let request = Request::builder("{{API_ROOT}}/foobar", RequestMethod::Get)
            .header(
                &Field::builder()
                    .key("X-Api-Key")
                    .value("{{API_TOKEN}}")
                    .build(),
            )
            .variable(
                &Field::builder()
                    .key("API_ROOT")
                    .value("https://api.example.com")
                    .build(),
            )
            .variable(
                &Field::builder()
                    .key("API_TOKEN")
                    .value("123412341234")
                    .build(),
            )
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("curl -X GET 'https://api.example.com/foobar'"),
            "Expected {} to contain the rendered URL",
            output
        );
        assert!(
            output.contains("-H 'X-Api-Key: 123412341234'"),
            "Expected {} to contain the header",
            output
        );
    }

    #[test]
    fn test_renders_basic_auth() {
        let basic_auth = RequestAuthenticationBasic::builder()
            .username("admin")
            .password("{{PASSWORD}}")
            .build();
        let auth = RequestAuthentication::builder()
            .basic_auth(&basic_auth)
            .build();
        let request = Request::builder("http://localhost:3000/foobar", RequestMethod::Get)
            .variable(
                &Field::builder()
                    .key("PASSWORD")
                    .value("12341234")
                    .masked(true)
                    .build(),
            )
            .with_auth(auth)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("-H 'Authorization: Basic YWRtaW46MTIzNDEyMzQ="),
            "Expected {} to contain the encoded credentials",
            output
        );
    }

    #[test]
    fn test_renders_bearer_tokens() {
        let bearer_token = RequestAuthenticationBearer::builder()
            .token("secret_token")
            .build();
        let auth = RequestAuthentication::builder()
            .bearer_token(&bearer_token)
            .build();
        let request = Request::builder("http://localhost:3000/foobar", RequestMethod::Get)
            .with_auth(auth)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("-H 'Authorization: Bearer secret_token"),
            "Expected {} to contain the encoded credentials",
            output
        );
    }

    #[test]
    fn test_renders_urlencoded() {
        let urlencoded = RequestBodyUrlencoded::builder()
            .field(&Field::builder().key("user_id").value("100").build())
            .field(
                &Field::builder()
                    .key("category")
                    .value("home office")
                    .build(),
            )
            .field(
                &Field::builder()
                    .key("ignore_me")
                    .value("please")
                    .active(false)
                    .build(),
            )
            .field(&Field::builder().key("is_admin").value("{{ADMIN}}").build())
            .build();
        let body = RequestBody::builder().urlencoded(&urlencoded).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .variable(&Field::builder().key("ADMIN").value("true").build())
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("-d 'user_id=100&category=home+office&is_admin=true'"),
            "Expected {} to contain the encoded body",
            output
        );
    }

    #[test]
    fn test_renders_multipart() {
        let multipart = RequestBodyMultipart::builder()
            .field(&Field::builder().key("user_id").value("100").build())
            .field(
                &Field::builder()
                    .key("category")
                    .value("home office")
                    .build(),
            )
            .field(
                &Field::builder()
                    .key("ignore_me")
                    .value("please")
                    .active(false)
                    .build(),
            )
            .field(&Field::builder().key("is_admin").value("{{ADMIN}}").build())
            .build();
        let body = RequestBody::builder().multipart(&multipart).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .variable(&Field::builder().key("ADMIN").value("true").build())
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("-F 'user_id=100'"),
            "Expected {} to contain the encoded body for user_id",
            output
        );
        assert!(
            output.contains("-F 'category=home office'"),
            "Expected {} to contain the encoded body for category",
            output
        );
        assert!(
            output.contains("-F 'is_admin=true'"),
            "Expected {} to contain the encoded body for is_admin",
            output
        );
        assert!(
            !output.contains("-H 'Content-Type: multipart/form-data; boundary="),
            "Expected {} not to contain a multipart header",
            output
        );
    }

    #[test]
    fn test_renders_json() {
        let json = RequestBodyRaw::builder(RequestBodyRawType::Json)
            .payload(r#"{"hello": "world"}"#)
            .build();
        let body = RequestBody::builder().raw(&json).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains(r#"-d '{"hello":"world"}'"#),
            "Expected {} to contain the encoded body",
            output
        );
        assert!(
            output.contains("-H 'Content-Type: application/json'"),
            "Expected {} to contain the default header",
            output
        );
    }

    #[test]
    fn test_renders_json_with_custom_content_type() {
        let json = RequestBodyRaw::builder(RequestBodyRawType::Json)
            .payload(r#"{"hello": "world"}"#)
            .build();
        let body = RequestBody::builder().raw(&json).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .header(
                &Field::builder()
                    .key("Content-Type")
                    .value("application/activity+json")
                    .build(),
            )
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains(r#"-d '{"hello":"world"}'"#),
            "Expected {} to contain the encoded body",
            output
        );
        assert!(
            output.contains("-H 'Content-Type: application/activity+json'"),
            "Expected {} to contain the custom header",
            output
        );
    }

    #[test]
    fn test_renders_xml() {
        let xml = RequestBodyRaw::builder(RequestBodyRawType::Xml)
            .payload(r#"<?xml version="1.0" ?><envelope />"#)
            .build();
        let body = RequestBody::builder().raw(&xml).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains(r#"-d '<?xml version="1.0" ?><envelope />'"#),
            "Expected {} to contain the encoded body",
            output
        );
        assert!(
            output.contains("-H 'Content-Type: application/xml'"),
            "Expected {} to contain the default header",
            output
        );
    }

    #[test]
    fn test_renders_xml_with_custom_content_type() {
        let xml = RequestBodyRaw::builder(RequestBodyRawType::Xml)
            .payload(r#"<?xml version="1.0" ?><envelope />"#)
            .build();
        let body = RequestBody::builder().raw(&xml).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .header(
                &Field::builder()
                    .key("Content-Type")
                    .value("application/atom+xml")
                    .build(),
            )
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains(r#"-d '<?xml version="1.0" ?><envelope />'"#),
            "Expected {} to contain the encoded body",
            output
        );
        assert!(
            output.contains("-H 'Content-Type: application/atom+xml'"),
            "Expected {} to contain the custom header",
            output
        );
    }

    #[test]
    fn test_renders_octet_stream() {
        let os = RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
            .payload("ID,Date,Description,Ready")
            .build();
        let body = RequestBody::builder().raw(&os).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("-d 'ID,Date,Description,Ready'"),
            "Expected {} to contain the encoded body",
            output
        );
        assert!(
            output.contains("-H 'Content-Type: application/octet-stream'"),
            "Expected {} to contain the default header",
            output
        );
    }

    #[test]
    fn test_renders_octet_stream_with_custom_content_type() {
        let os = RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
            .payload("ID,Date,Description,Ready")
            .build();
        let body = RequestBody::builder().raw(&os).build();
        let request = Request::builder("https://localhost:3000/foobar", RequestMethod::Post)
            .header(
                &Field::builder()
                    .key("Content-Type")
                    .value("text/csv")
                    .build(),
            )
            .with_body(body)
            .build();
        let output = super::export_as_curl(&request).expect("Should have been OK");
        assert!(
            output.contains("-d 'ID,Date,Description,Ready'"),
            "Expected {} to contain the encoded body",
            output
        );
        assert!(
            output.contains("-H 'Content-Type: text/csv'"),
            "Expected {} to contain the custom header",
            output
        );
    }
}
