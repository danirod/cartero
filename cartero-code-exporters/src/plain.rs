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

use cartero_objects::{
    Field, FieldTable, Request as ObjectRequest, RequestAuthenticationBasic,
    RequestAuthenticationBearer, RequestAuthenticationData, RequestAuthenticationDataExt,
    RequestBody, RequestBodyDataExt, RequestBodyFile, RequestBodyMultipart, RequestBodyRaw,
    RequestBodyUrlencoded,
};
use gio::prelude::ListModelExtManual;
use glib::object::Cast;

#[derive(Eq, PartialEq, Clone)]
pub(crate) enum Auth {
    None,
    BasicAuth { username: String, password: String },
    BearerToken { token: String },
}

#[derive(Eq, PartialEq, Clone)]
pub(crate) enum Body {
    Urlencoded {
        values: Vec<(String, String)>,
        urlencoded: String,
    },
    Multipart {
        inline: Vec<(String, String)>,
    },
    Raw {
        payload: String,
    },
    File {
        path: String,
    },
}

impl Body {
    pub fn extract(request: &RequestBody) -> (Option<Self>, Vec<(String, String)>) {
        match request.body_type() {
            cartero_objects::RequestBodyType::UrlEncoded => {
                Self::extract_urlencoded(&request.urlencoded().expect("Where is my urlencoded?"))
            }
            cartero_objects::RequestBodyType::Multipart => {
                Self::extract_multipart(&request.multipart().expect("Where is my multipart?"))
            }
            cartero_objects::RequestBodyType::Raw => {
                Self::extract_raw(&request.raw().expect("Where is my raw?"))
            }
            cartero_objects::RequestBodyType::File => {
                Self::extract_file(&request.file().expect("Where is my file?"))
            }
            _ => (None, vec![]),
        }
    }

    fn extract_urlencoded(
        urlencoded: &RequestBodyUrlencoded,
    ) -> (Option<Self>, Vec<(String, String)>) {
        let values = extract_table(&urlencoded.params());
        let encoded_values = serde_urlencoded::to_string(&values).unwrap();
        let body = Self::Urlencoded {
            values,
            urlencoded: encoded_values,
        };
        (Some(body), vec![])
    }

    fn extract_multipart(
        multipart: &RequestBodyMultipart,
    ) -> (Option<Self>, Vec<(String, String)>) {
        let values = extract_table(&multipart.params());
        let body = Self::Multipart { inline: values };
        (Some(body), vec![])
    }

    fn extract_raw(raw: &RequestBodyRaw) -> (Option<Self>, Vec<(String, String)>) {
        let payload = raw.payload();
        let headers = raw.rendered_headers();
        let body = Self::Raw { payload };
        (Some(body), headers)
    }

    fn extract_file(file: &RequestBodyFile) -> (Option<Self>, Vec<(String, String)>) {
        let path = file.path();
        let headers = match file.content_type() {
            None => {
                vec![]
            }
            Some(content_type) => match content_type.trim() {
                "" => vec![],
                anything => vec![("Content-Type".to_string(), anything.to_string())],
            },
        };
        let body = Self::File { path };
        (Some(body), headers)
    }
}

#[derive(Eq, PartialEq, Clone)]
pub(crate) struct Request {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub auth: Option<Auth>,
    pub body: Option<Body>,
}

fn extract_table(table: &FieldTable) -> Vec<(String, String)> {
    let mut headers = table
        .iter::<Field>()
        .filter_map(|row| row.ok().take_if(|field| field.active()))
        .map(|field| (field.key(), field.value()))
        .collect::<Vec<(String, String)>>();
    headers.sort_by_key(|(k, _)| k.to_string());
    headers
}

impl From<ObjectRequest> for Request {
    fn from(value: ObjectRequest) -> Self {
        let (body, body_headers) = Body::extract(&value.body());
        let headers = extract_table(&value.headers());

        let mut headers = headers
            .into_iter()
            .chain(body_headers)
            .collect::<Vec<(String, String)>>();
        headers.sort_by_key(|(k, _)| k.to_string());

        Self {
            url: value.url(),
            method: value.method().to_string(),
            headers,
            auth: value.authentication().auth_data().map(|data| data.into()),
            body,
        }
    }
}

impl From<RequestAuthenticationData> for Auth {
    fn from(value: RequestAuthenticationData) -> Self {
        match value.auth_type() {
            cartero_objects::RequestAuthenticationType::BasicAuth => {
                let basic = value
                    .downcast::<RequestAuthenticationBasic>()
                    .expect("No basic?");
                Self::BasicAuth {
                    username: basic.username(),
                    password: basic.password(),
                }
            }
            cartero_objects::RequestAuthenticationType::BearerToken => {
                let bearer = value
                    .downcast::<RequestAuthenticationBearer>()
                    .expect("No bearer?");
                Self::BearerToken {
                    token: bearer.token(),
                }
            }
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction() {
        let request = ObjectRequest::builder(
            "https://www.example.com/foobar",
            cartero_objects::RequestMethod::Post,
        )
        .header(
            &Field::builder()
                .key("X-Api-Key")
                .value("123412341234")
                .build(),
        )
        .build();
        let parsed_request = Request::from(request);
        assert_eq!(parsed_request.url, "https://www.example.com/foobar");
        assert_eq!(parsed_request.method, "POST");
        assert_eq!(
            parsed_request.headers,
            vec![("X-Api-Key".to_string(), "123412341234".to_string()),]
        );
    }

    #[test]
    fn test_extraction_ignores_disabled_variables() {
        let request = ObjectRequest::builder(
            "https://www.example.com/foobar",
            cartero_objects::RequestMethod::Post,
        )
        .header(
            &Field::builder()
                .key("X-Api-Key")
                .value("123412341234")
                .active(false)
                .build(),
        )
        .build();
        let parsed_request = Request::from(request);
        assert_eq!(parsed_request.url, "https://www.example.com/foobar");
        assert_eq!(parsed_request.method, "POST");
        assert_eq!(parsed_request.headers, vec![]);
    }
}
