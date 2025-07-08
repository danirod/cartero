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

use cartero_interop::{FileLoadError, FileOpResult, FileWarningTag};
use cartero_objects::{
    Field, FieldTable, Request, RequestAuthenticationBasic, RequestAuthenticationBearer,
    RequestAuthenticationType, RequestBody, RequestMethod,
};
use gio::prelude::{ListModelExt, ListModelExtManual};
use glib::Object;
use serde::{Deserialize, Serialize};

use crate::{
    authorization_value::AuthorizationValue,
    field_table_value::FieldTableValue,
    field_value::FieldValue,
    payload_value::{PayloadRawFormat, PayloadValue, PayloadValueOrString},
    query_value::QueryValue,
};

const VERSION_GENERATION: usize = 1;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RequestValue {
    version: usize,
    url: String,
    method: String,
    body: Option<PayloadValueOrString>,
    #[serde(serialize_with = "crate::serializer::alphabetical_field_table")]
    headers: Option<FieldTableValue<FieldValue>>,
    #[serde(serialize_with = "crate::serializer::alphabetical_field_table")]
    variables: Option<FieldTableValue<FieldValue>>,
    authorization: Option<AuthorizationValue>,
    #[serde(
        rename = "inactive-params",
        serialize_with = "crate::serializer::alphabetical_field_table"
    )]
    inactive_params: Option<FieldTableValue<QueryValue>>,
}

impl TryFrom<RequestValue> for FileOpResult<Request> {
    type Error = FileLoadError;

    fn try_from(value: RequestValue) -> Result<Self, Self::Error> {
        if value.version < 1 {
            // TODO: in this case, shouldn't there be a general corruption issue?
            return Err(FileLoadError::SchemaTooNew);
        }
        if value.version > VERSION_GENERATION {
            return Err(FileLoadError::SchemaTooNew);
        }

        let method = match RequestMethod::try_from(value.method.as_str()) {
            Ok(method) => Ok(method),
            Err(_) => Err(FileWarningTag::InvalidHttpVerb(value.method.clone())),
        };

        let mut tags = vec![];
        if let Err(method_tag) = method.as_ref() {
            tags.push(method_tag.clone());
        };

        let headers = value.headers.map(FieldTable::from).unwrap_or_default();
        let variables = value.variables.map(FieldTable::from).unwrap_or_default();
        let inactive_params = value
            .inactive_params
            .map(FieldTable::from)
            .unwrap_or_default();
        let params = craft_parameters_table(&value.url, &inactive_params);

        let request: Request = Object::builder()
            .property("url", value.url)
            .property("method", method.unwrap_or_default())
            .property("headers", headers)
            .property("variables", variables)
            .property("params", params)
            .build();

        if let Some(auth) = value.authorization {
            match auth {
                AuthorizationValue::Basic { username, password } => {
                    request
                        .authentication()
                        .set_auth_type(RequestAuthenticationType::BasicAuth);
                    request
                        .authentication()
                        .set_auth_data(Some(RequestAuthenticationBasic::new(username, password)));
                }
                AuthorizationValue::Bearer { token } => {
                    request
                        .authentication()
                        .set_auth_type(RequestAuthenticationType::BearerToken);
                    request
                        .authentication()
                        .set_auth_data(Some(RequestAuthenticationBearer::new(token)));
                }
            }
        }

        if let Some(body) = value.body {
            let payload = match body {
                PayloadValueOrString::Raw(raw) => PayloadValue::Raw {
                    format: Some(PayloadRawFormat::OctetStream),
                    body: raw,
                },
                PayloadValueOrString::Structured(obj) => obj,
            };
            let body = RequestBody::from(payload);
            request.body().set_body_type(body.body_type());
            request.body().set_body_data(body.body_data());
        }

        Ok(FileOpResult::new(request, &tags))
    }
}

impl From<Request> for RequestValue {
    fn from(value: Request) -> Self {
        let url = value.url().clone();
        let method = value.method().to_string();
        let body = match value.body().body_type() {
            cartero_objects::RequestBodyType::None => None,
            _ => Some(PayloadValue::from(value.body())),
        };
        let body = body.map(PayloadValueOrString::Structured);
        let headers = if value.headers().n_items() > 0 {
            Some(value.headers().into())
        } else {
            None
        };
        let variables = if value.variables().n_items() > 0 {
            Some(value.variables().into())
        } else {
            None
        };
        let authorization = match value.authentication().auth_type() {
            cartero_objects::RequestAuthenticationType::None => None,
            _ => match AuthorizationValue::try_from(value.authentication()) {
                Ok(result) => Some(result),
                Err(_) => None,
            },
        };

        let inactive_params = value
            .params()
            .iter::<Field>()
            .map(|map| map.unwrap())
            .filter(|param| !param.active())
            .collect::<Vec<Field>>();
        let inactive_params = if !inactive_params.is_empty() {
            let table = FieldTable::from_iter(inactive_params);
            Some(table.into())
        } else {
            None
        };

        Self {
            version: VERSION_GENERATION,
            url,
            method,
            body,
            headers,
            variables,
            authorization,
            inactive_params,
        }
    }
}

fn craft_parameters_table(url: &str, inactive: &FieldTable) -> FieldTable {
    let inactive_iter = inactive.iter::<Field>().filter_map(|obj| obj.ok());
    extract_queryparams(url)
        .iter()
        .map(|(key, value)| Field::from((key, value)))
        .chain(inactive_iter)
        .collect::<FieldTable>()
}

fn extract_queryparams(url: &str) -> Vec<(String, String)> {
    let parts = url.split("?").collect::<Vec<&str>>();
    if parts.len() < 2 {
        vec![]
    } else {
        let combined = parts[1..].join("?");
        let params = form_urlencoded::parse(combined.as_bytes());
        params
            .into_iter()
            .map(|(key, value)| (String::from(key), String::from(value)))
            .collect::<_>()
    }
}

#[cfg(test)]
mod tests {
    use cartero_interop::{FileLoadError, FileOpResult, FileWarningTag};
    use cartero_objects::{
        Field, FieldTable, Request, RequestAuthenticationBearer, RequestAuthenticationType,
        RequestBodyRaw, RequestBodyType, RequestMethod,
    };
    use gio::prelude::ListModelExt;
    use glib::Object;

    use crate::{authorization_value::AuthorizationValue, payload_value::PayloadValueOrString};

    use super::RequestValue;

    #[test]
    fn converts_from_request() {
        let request: Request = Object::builder()
            .property("url", "https://www.example.com/index.php")
            .property("method", RequestMethod::Post)
            .build();
        let value = RequestValue::from(request);
        assert_eq!(value.version, 1);
        assert_eq!(value.url, "https://www.example.com/index.php");
        assert_eq!(value.method, "POST");
        assert!(value.headers.is_none());
        assert!(value.body.is_none());
        assert!(value.variables.is_none());
        assert!(value.authorization.is_none());
        assert!(value.inactive_params.is_none());
    }

    #[test]
    fn converts_from_request_with_headers() {
        let header1 = Field::from(("User-Agent", "Mozilla/5.0"));
        let header2 = Field::from(("Accept", "text/html"));
        let headers = FieldTable::from_iter([header1, header2]);
        let request: Request = Object::builder()
            .property("url", "https://www.example.com/index.php")
            .property("method", RequestMethod::Delete)
            .property("headers", headers)
            .build();
        let value = RequestValue::from(request);
        assert_eq!(value.version, 1);
        assert_eq!(value.url, "https://www.example.com/index.php");
        assert_eq!(value.method, "DELETE");
        assert!(value.headers.is_some_and(|headers| {
            assert_eq!(2, headers.len());
            assert!(headers.get("User-Agent").is_some_and(|variable| {
                let fields = variable.to_fields("User-Agent");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].key(), "User-Agent");
                assert_eq!(fields[0].value(), "Mozilla/5.0");
                assert!(fields[0].active());
                assert!(!fields[0].masked());
                true
            }));
            assert!(headers.get("Accept").is_some_and(|variable| {
                let fields = variable.to_fields("Accept");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].key(), "Accept");
                assert_eq!(fields[0].value(), "text/html");
                assert!(fields[0].active());
                assert!(!fields[0].masked());
                true
            }));
            true
        }));
        assert!(value.body.is_none());
        assert!(value.variables.is_none());
        assert!(value.authorization.is_none());
        assert!(value.inactive_params.is_none());
    }

    #[test]
    fn converts_from_request_with_body() {
        let request: Request = Object::builder()
            .property("url", "https://www.example.com/index.php")
            .property("method", RequestMethod::Put)
            .build();
        request.body().set_body_type(RequestBodyType::Raw);
        request.body().set_body_data(Some(RequestBodyRaw::new(
            cartero_objects::RequestBodyRawType::OctetStream,
            "hello world".as_bytes(),
        )));
        let value = RequestValue::from(request);
        assert_eq!(value.version, 1);
        assert_eq!(value.url, "https://www.example.com/index.php");
        assert_eq!(value.method, "PUT");
        assert!(value.headers.is_none());
        let PayloadValueOrString::Structured(body) = value.body.unwrap() else {
            panic!("Payload is not structured");
        };
        match body {
            crate::payload_value::PayloadValue::Raw { format, body } => {
                assert_eq!(
                    format,
                    Some(crate::payload_value::PayloadRawFormat::OctetStream)
                );
                assert_eq!(body, "hello world");
                true
            }
            _ => panic!("Body not of valid type"),
        };
        assert!(value.variables.is_none());
        assert!(value.authorization.is_none());
        assert!(value.inactive_params.is_none());
    }

    #[test]
    fn converts_from_request_with_variables() {
        let variable1 = Field::from(("API_ROOT", "http://api.example.com"));
        let variable2 = Field::from(("API_KEY", "key1234"));
        let variables = FieldTable::from_iter([variable1, variable2]);
        let request: Request = Object::builder()
            .property("url", "{{API_ROOT}}/users?api_key={{API_KEY}}")
            .property("method", RequestMethod::Get)
            .property("variables", variables)
            .build();
        let value = RequestValue::from(request);
        assert_eq!(value.version, 1);
        assert_eq!(value.url, "{{API_ROOT}}/users?api_key={{API_KEY}}");
        assert_eq!(value.method, "GET");
        assert!(value.headers.is_none());
        assert!(value.body.is_none());
        assert!(value.variables.is_some_and(|variables| {
            assert_eq!(2, variables.len());
            assert!(variables.get("API_ROOT").is_some_and(|variable| {
                let fields = variable.to_fields("API_ROOT");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].key(), "API_ROOT");
                assert_eq!(fields[0].value(), "http://api.example.com");
                assert!(fields[0].active());
                assert!(!fields[0].masked());
                true
            }));
            assert!(variables.get("API_KEY").is_some_and(|variable| {
                let fields = variable.to_fields("API_KEY");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].key(), "API_KEY");
                assert_eq!(fields[0].value(), "key1234");
                assert!(fields[0].active());
                assert!(!fields[0].masked());
                true
            }));
            true
        }));
        assert!(value.authorization.is_none());
        assert!(value.inactive_params.is_none());
    }

    #[test]
    fn converts_from_request_with_authorization() {
        let request: Request = Object::builder()
            .property("url", "https://www.example.com/index.php")
            .property("method", RequestMethod::Put)
            .build();
        request
            .authentication()
            .set_auth_type(RequestAuthenticationType::BearerToken);
        request
            .authentication()
            .set_auth_data(Some(RequestAuthenticationBearer::new("token1234")));
        let value = RequestValue::from(request);
        assert_eq!(value.version, 1);
        assert_eq!(value.url, "https://www.example.com/index.php");
        assert_eq!(value.method, "PUT");
        assert!(value.headers.is_none());
        assert!(value.body.is_none());
        assert!(value.variables.is_none());
        assert!(value.authorization.is_some_and(|auth| {
            match auth {
                crate::authorization_value::AuthorizationValue::Bearer { token } => {
                    assert_eq!(token, "token1234");
                    true
                }
                _ => panic!("invalid authentication type"),
            }
        }));
        assert!(value.inactive_params.is_none());
    }

    #[test]
    fn converts_from_request_with_disabled_headers() {
        let param1 = Field::from(("page", "2"));
        let param2 = Field::from(("per_page", "15"));
        let param3 = Field::from(("sort", "-created"));
        param3.set_active(false);
        let params = FieldTable::from_iter([param1, param2, param3]);
        let request: Request = Object::builder()
            .property("url", "http://api.example.com/books?page=2&per_page=15")
            .property("method", RequestMethod::Get)
            .property("params", params)
            .build();
        let value = RequestValue::from(request);
        assert_eq!(value.version, 1);
        assert_eq!(value.url, "http://api.example.com/books?page=2&per_page=15");
        assert_eq!(value.method, "GET");
        assert!(value.headers.is_none());
        assert!(value.body.is_none());
        assert!(value.variables.is_none());
        assert!(value.authorization.is_none());
        assert!(value.inactive_params.is_some_and(|params| {
            assert_eq!(1, params.len());
            assert!(params.get("sort").is_some_and(|variable| {
                let fields = variable.to_fields("sort");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].key(), "sort");
                assert_eq!(fields[0].value(), "-created");
                assert!(!fields[0].active());
                assert!(!fields[0].masked());
                true
            }));
            true
        }));
    }

    #[test]
    fn converts_to_request_with_invalid_version() {
        let value = RequestValue {
            version: 0,
            url: "https://www.example.com/index.html".to_string(),
            method: "GET".to_string(),
            authorization: None,
            body: None,
            headers: None,
            inactive_params: None,
            variables: None,
        };
        let result: Result<FileOpResult<Request>, FileLoadError> = value.try_into();
        assert!(result.is_err_and(|e| match e {
            FileLoadError::SchemaTooNew => true,
            _ => false,
        }));
    }

    #[test]
    fn converts_to_request_with_version_too_new() {
        let value = RequestValue {
            version: 2,
            url: "https://www.example.com/index.html".to_string(),
            method: "GET".to_string(),
            authorization: None,
            body: None,
            headers: None,
            inactive_params: None,
            variables: None,
        };
        let result: Result<FileOpResult<Request>, FileLoadError> = value.try_into();
        assert!(result.is_err_and(|e| match e {
            FileLoadError::SchemaTooNew => true,
            _ => false,
        }));
    }

    #[test]
    fn converts_to_request_with_invalid_method() {
        let value = RequestValue {
            version: 1,
            url: "https://www.example.com/index.html".to_string(),
            method: "HELLO".to_string(),
            authorization: None,
            body: None,
            headers: None,
            inactive_params: None,
            variables: None,
        };
        let result: Result<FileOpResult<Request>, FileLoadError> = value.try_into();
        assert!(result.is_ok_and(|result| {
            assert_eq!(1, result.warnings().len());
            assert!(match &result.warnings()[0] {
                FileWarningTag::InvalidHttpVerb(v) => v == "HELLO",
            });
            assert_eq!(result.object().url(), "https://www.example.com/index.html");
            assert_eq!(result.object().method(), RequestMethod::Get);
            true
        }));
    }

    #[test]
    fn converts_to_request() {
        let value = RequestValue {
            version: 1,
            url: "https://www.example.com/index.html".to_string(),
            method: "GET".to_string(),
            authorization: None,
            body: None,
            headers: None,
            inactive_params: None,
            variables: None,
        };
        let result: Result<FileOpResult<Request>, FileLoadError> = value.try_into();
        assert!(result.is_ok_and(|result| {
            assert!(result.warnings().is_empty());
            let request = result.object();
            assert_eq!(request.url(), "https://www.example.com/index.html");
            assert_eq!(request.method(), RequestMethod::Get);
            assert_eq!(
                request.authentication().auth_type(),
                RequestAuthenticationType::None
            );
            assert_eq!(request.body().body_type(), RequestBodyType::None);
            assert_eq!(request.headers().n_items(), 0);
            assert_eq!(request.variables().n_items(), 0);
            assert_eq!(request.params().n_items(), 0);
            true
        }));
    }

    #[test]
    fn converts_to_request_with_basic_auth() {
        let value = RequestValue {
            version: 1,
            url: "https://www.example.com/index.html".to_string(),
            method: "GET".to_string(),
            authorization: Some(AuthorizationValue::Basic {
                username: "root".to_string(),
                password: "password".to_string(),
            }),
            body: None,
            headers: None,
            inactive_params: None,
            variables: None,
        };
        let result: Result<FileOpResult<Request>, FileLoadError> = value.try_into();
        assert!(result.is_ok_and(|result| {
            assert!(result.warnings().is_empty());
            let request = result.object();
            assert_eq!(request.url(), "https://www.example.com/index.html");
            assert_eq!(request.method(), RequestMethod::Get);
            assert_eq!(
                request.authentication().auth_type(),
                RequestAuthenticationType::BasicAuth
            );
            let basic_auth = request.authentication().basic_auth().unwrap();
            assert_eq!(basic_auth.username(), "root");
            assert_eq!(basic_auth.password(), "password");
            assert_eq!(request.body().body_type(), RequestBodyType::None);
            assert_eq!(request.headers().n_items(), 0);
            assert_eq!(request.variables().n_items(), 0);
            assert_eq!(request.params().n_items(), 0);
            true
        }));
    }

    #[test]
    fn converts_to_request_with_bearer_auth() {
        let value = RequestValue {
            version: 1,
            url: "https://www.example.com/index.html".to_string(),
            method: "GET".to_string(),
            authorization: Some(AuthorizationValue::Bearer {
                token: "token".to_string(),
            }),
            body: None,
            headers: None,
            inactive_params: None,
            variables: None,
        };
        let result: Result<FileOpResult<Request>, FileLoadError> = value.try_into();
        assert!(result.is_ok_and(|result| {
            assert!(result.warnings().is_empty());
            let request = result.object();
            assert_eq!(request.url(), "https://www.example.com/index.html");
            assert_eq!(request.method(), RequestMethod::Get);
            assert_eq!(
                request.authentication().auth_type(),
                RequestAuthenticationType::BearerToken
            );
            let bearer_token = request.authentication().bearer_token().unwrap();
            assert_eq!(bearer_token.token(), "token");
            assert_eq!(request.body().body_type(), RequestBodyType::None);
            assert_eq!(request.headers().n_items(), 0);
            assert_eq!(request.variables().n_items(), 0);
            assert_eq!(request.params().n_items(), 0);
            true
        }));
    }
}
