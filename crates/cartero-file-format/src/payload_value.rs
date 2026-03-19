// Copyright 2024-2026 the Cartero authors
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
    FieldTable, RequestBody, RequestBodyFile, RequestBodyMultipart, RequestBodyRaw,
    RequestBodyRawType, RequestBodyType, RequestBodyUrlencoded,
};
use serde::{Deserialize, Serialize};

use crate::{field_table_value::FieldTableValue, field_value::FieldValue};

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub(crate) enum PayloadRawFormat {
    #[serde(rename = "octet-stream")]
    OctetStream,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "xml")]
    Xml,
}

impl From<RequestBodyRawType> for PayloadRawFormat {
    fn from(value: RequestBodyRawType) -> Self {
        match value {
            RequestBodyRawType::OctetStream => Self::OctetStream,
            RequestBodyRawType::Json => Self::Json,
            RequestBodyRawType::Xml => Self::Xml,
        }
    }
}

impl From<PayloadRawFormat> for RequestBodyRawType {
    fn from(value: PayloadRawFormat) -> Self {
        match value {
            PayloadRawFormat::Json => Self::Json,
            PayloadRawFormat::OctetStream => Self::OctetStream,
            PayloadRawFormat::Xml => Self::Xml,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum PayloadValue {
    #[serde(rename = "urlencoded")]
    UrlEncoded {
        #[serde(serialize_with = "crate::serializer::alphabetical_field_table")]
        variables: Option<FieldTableValue<FieldValue>>,
    },
    #[serde(rename = "multipart")]
    Multipart {
        #[serde(serialize_with = "crate::serializer::alphabetical_field_table")]
        variables: Option<FieldTableValue<FieldValue>>,
    },
    #[serde(rename = "raw")]
    Raw {
        format: Option<PayloadRawFormat>,
        body: String,
    },
    #[serde(rename = "file")]
    File {
        path: String,
        content_type: Option<String>,
    },
}

impl From<RequestBody> for PayloadValue {
    fn from(value: RequestBody) -> Self {
        match value.body_type() {
            RequestBodyType::None => {
                panic!("please don't convert from RequestBodyType::None to a PayloadValue");
            }
            RequestBodyType::UrlEncoded => {
                let urlencoded = value.urlencoded().unwrap();
                Self::UrlEncoded {
                    variables: Some(urlencoded.params().into()),
                }
            }
            RequestBodyType::Multipart => {
                let multipart = value.multipart().unwrap();
                Self::Multipart {
                    variables: Some(multipart.params().into()),
                }
            }
            RequestBodyType::Raw => {
                let raw = value.raw().unwrap();
                Self::Raw {
                    format: Some(raw.payload_type().into()),
                    body: raw.payload(),
                }
            }
            RequestBodyType::File => {
                let file = value.file().unwrap();
                let content_type = file
                    .content_type()
                    .and_then(|ct| if ct.trim().is_empty() { None } else { Some(ct) });
                Self::File {
                    path: file.path(),
                    content_type,
                }
            }
        }
    }
}

impl From<PayloadValue> for RequestBody {
    fn from(value: PayloadValue) -> Self {
        match value {
            PayloadValue::Raw { format, body } => {
                let parsed_format: RequestBodyRawType =
                    format.unwrap_or(PayloadRawFormat::OctetStream).into();
                let parsed_body = RequestBodyRaw::new(parsed_format, &body);
                RequestBody::new(RequestBodyType::Raw, Some(parsed_body))
            }
            PayloadValue::Multipart { variables } => {
                let parsed_variables = variables.map(FieldTable::from).unwrap_or_default();
                let parsed_body = RequestBodyMultipart::from_table(&parsed_variables);
                RequestBody::new(RequestBodyType::Multipart, Some(parsed_body))
            }
            PayloadValue::UrlEncoded { variables } => {
                let parsed_variables = variables.map(FieldTable::from).unwrap_or_default();
                let parsed_body = RequestBodyUrlencoded::from_table(&parsed_variables);
                RequestBody::new(RequestBodyType::UrlEncoded, Some(parsed_body))
            }
            PayloadValue::File { path, content_type } => {
                let file = RequestBodyFile::new(path, content_type);
                RequestBody::new(RequestBodyType::File, Some(file))
            }
        }
    }
}

/// We actually never encode into this, but this allows to read TOML files where
/// the payload is just a string. Some of the early builds of Cartero accepted
/// raw strings rather than objects.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum PayloadValueOrString {
    Raw(String),
    Structured(PayloadValue),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use cartero_objects::Field;
    use gio::prelude::{ListModelExt, ListModelExtManual};

    use crate::field_table_value::FieldValueTableRow;

    use super::*;

    #[test]
    fn deserialize_url_encoded() {
        let params = [
            (
                "user_id".to_string(),
                FieldValueTableRow::Unique(FieldValue::Simple("1234".to_string())),
            ),
            (
                "group_id".to_string(),
                FieldValueTableRow::Unique(FieldValue::Simple("200".to_string())),
            ),
        ];
        let params = FieldTableValue::new(HashMap::from(params));
        let body = PayloadValue::UrlEncoded {
            variables: Some(params),
        };

        let parsed = RequestBody::from(body);
        assert_eq!(parsed.body_type(), RequestBodyType::UrlEncoded);
        let body = parsed.urlencoded().unwrap();

        let body_params = body.params();
        assert_eq!(2, body_params.n_items());
        assert!(
            body_params
                .iter::<Field>()
                .any(|f| f.is_ok_and(|f| f.key() == "group_id" && f.value() == "200"))
        );
        assert!(
            body_params
                .iter::<Field>()
                .any(|f| f.is_ok_and(|f| f.key() == "user_id" && f.value() == "1234"))
        );
    }

    #[test]
    fn serialize_url_encoded() {
        let param1 = Field::from(("user_id", "1234"));
        let param2 = Field::from(("category", "1"));
        let param3 = Field::from(("password", "p4ssw0rd"));
        param3.set_masked(true);
        let param4 = Field::from(("group_id", "100"));
        param4.set_active(false);
        let param5 = Field::from(("category", "2"));
        let table = FieldTable::from_iter([param1, param2, param3, param4, param5]);
        let body = RequestBody::new(
            RequestBodyType::UrlEncoded,
            Some(RequestBodyUrlencoded::from_table(&table)),
        );

        let serial = PayloadValue::from(body);
        let PayloadValue::UrlEncoded { variables } = serial else {
            panic!("Not the expected type");
        };
        let variables = variables.unwrap();

        match variables.get("user_id").unwrap() {
            FieldValueTableRow::Unique(unique) => match unique {
                FieldValue::Simple(user_id) => assert_eq!(user_id, "1234"),
                _ => panic!("Not the expected type"),
            },
            _ => panic!("Not the expected type"),
        };

        match variables.get("password").unwrap() {
            FieldValueTableRow::Unique(unique) => match unique {
                FieldValue::Complex {
                    value,
                    active,
                    secret,
                } => {
                    assert_eq!(value, "p4ssw0rd");
                    assert!(*active);
                    assert!(*secret);
                }
                _ => panic!("Not the expected type"),
            },
            _ => panic!("Not the expected type"),
        };

        match variables.get("group_id").unwrap() {
            FieldValueTableRow::Unique(unique) => match unique {
                FieldValue::Complex {
                    value,
                    active,
                    secret,
                } => {
                    assert_eq!(value, "100");
                    assert!(!*active);
                    assert!(!*secret);
                }
                _ => panic!("Not the expected type"),
            },
            _ => panic!("Not the expected type"),
        };

        match variables.get("category").unwrap() {
            FieldValueTableRow::Multiple(multiple) => {
                assert_eq!(2, multiple.len());
                match &multiple[0] {
                    FieldValue::Simple(val) => assert_eq!(val, "1"),
                    _ => panic!("Not the expected type"),
                };
                match &multiple[1] {
                    FieldValue::Simple(val) => assert_eq!(val, "2"),
                    _ => panic!("Not the expected type"),
                };
            }
            _ => panic!("Not the expected type"),
        }
    }

    #[test]
    fn deserialize_multipart() {
        let params = [
            (
                "user_id".to_string(),
                FieldValueTableRow::Unique(FieldValue::Simple("1234".to_string())),
            ),
            (
                "group_id".to_string(),
                FieldValueTableRow::Unique(FieldValue::Simple("200".to_string())),
            ),
        ];
        let params = FieldTableValue::new(HashMap::from(params));
        let body = PayloadValue::Multipart {
            variables: Some(params),
        };

        let parsed = RequestBody::from(body);
        assert_eq!(parsed.body_type(), RequestBodyType::Multipart);
        let body = parsed.multipart().unwrap();

        let body_params = body.params();
        assert_eq!(2, body_params.n_items());
        assert!(
            body_params
                .iter::<Field>()
                .any(|f| f.is_ok_and(|f| f.key() == "group_id" && f.value() == "200"))
        );
        assert!(
            body_params
                .iter::<Field>()
                .any(|f| f.is_ok_and(|f| f.key() == "user_id" && f.value() == "1234"))
        );
    }

    #[test]
    fn serialize_multipart() {
        let param1 = Field::from(("user_id", "1234"));
        let param2 = Field::from(("category", "1"));
        let param3 = Field::from(("password", "p4ssw0rd"));
        param3.set_masked(true);
        let param4 = Field::from(("group_id", "100"));
        param4.set_active(false);
        let param5 = Field::from(("category", "2"));
        let table = FieldTable::from_iter([param1, param2, param3, param4, param5]);
        let body = RequestBody::new(
            RequestBodyType::Multipart,
            Some(RequestBodyMultipart::from_table(&table)),
        );

        let serial = PayloadValue::from(body);
        let PayloadValue::Multipart { variables } = serial else {
            panic!("Not the expected type");
        };
        let variables = variables.unwrap();

        match variables.get("user_id").unwrap() {
            FieldValueTableRow::Unique(unique) => match unique {
                FieldValue::Simple(user_id) => assert_eq!(user_id, "1234"),
                _ => panic!("Not the expected type"),
            },
            _ => panic!("Not the expected type"),
        };

        match variables.get("password").unwrap() {
            FieldValueTableRow::Unique(unique) => match unique {
                FieldValue::Complex {
                    value,
                    active,
                    secret,
                } => {
                    assert_eq!(value, "p4ssw0rd");
                    assert!(*active);
                    assert!(*secret);
                }
                _ => panic!("Not the expected type"),
            },
            _ => panic!("Not the expected type"),
        };

        match variables.get("group_id").unwrap() {
            FieldValueTableRow::Unique(unique) => match unique {
                FieldValue::Complex {
                    value,
                    active,
                    secret,
                } => {
                    assert_eq!(value, "100");
                    assert!(!*active);
                    assert!(!*secret);
                }
                _ => panic!("Not the expected type"),
            },
            _ => panic!("Not the expected type"),
        };

        match variables.get("category").unwrap() {
            FieldValueTableRow::Multiple(multiple) => {
                assert_eq!(2, multiple.len());
                match &multiple[0] {
                    FieldValue::Simple(val) => assert_eq!(val, "1"),
                    _ => panic!("Not the expected type"),
                };
                match &multiple[1] {
                    FieldValue::Simple(val) => assert_eq!(val, "2"),
                    _ => panic!("Not the expected type"),
                };
            }
            _ => panic!("Not the expected type"),
        }
    }

    #[test]
    fn deserialize_raw() {
        let body = PayloadValue::Raw {
            format: None,
            body: String::from("this is the content"),
        };
        let parsed = RequestBody::from(body);
        assert_eq!(parsed.body_type(), RequestBodyType::Raw);
        let body = parsed.raw().unwrap();
        assert_eq!(body.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(body.payload(), "this is the content");
    }

    #[test]
    fn deserialize_octet_stream() {
        let body = PayloadValue::Raw {
            format: Some(PayloadRawFormat::OctetStream),
            body: String::from("this is the content"),
        };
        let parsed = RequestBody::from(body);
        assert_eq!(parsed.body_type(), RequestBodyType::Raw);
        let body = parsed.raw().unwrap();
        assert_eq!(body.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(body.payload(), "this is the content");
    }

    #[test]
    fn serialize_octet_stream() {
        let body = RequestBody::new(
            RequestBodyType::Raw,
            Some(RequestBodyRaw::new(
                RequestBodyRawType::OctetStream,
                "this is the content",
            )),
        );

        let parsed = PayloadValue::from(body);
        let PayloadValue::Raw { format, body } = parsed else {
            panic!("Not the expected type");
        };
        assert_eq!(format.unwrap(), PayloadRawFormat::OctetStream);
        assert_eq!(body, "this is the content");
    }

    #[test]
    fn deserialize_json() {
        let body = PayloadValue::Raw {
            format: Some(PayloadRawFormat::Json),
            body: String::from("{\"result\": \"hello world\"}"),
        };
        let parsed = RequestBody::from(body);
        assert_eq!(parsed.body_type(), RequestBodyType::Raw);
        let body = parsed.raw().unwrap();
        assert_eq!(body.payload_type(), RequestBodyRawType::Json);
        assert_eq!(body.payload(), "{\"result\": \"hello world\"}");
    }

    #[test]
    fn serialize_json() {
        let body = RequestBody::new(
            RequestBodyType::Raw,
            Some(RequestBodyRaw::new(
                RequestBodyRawType::Json,
                r#"{"user_id": "200"}"#,
            )),
        );

        let parsed = PayloadValue::from(body);
        let PayloadValue::Raw { format, body } = parsed else {
            panic!("Not the expected type");
        };
        assert_eq!(format.unwrap(), PayloadRawFormat::Json);
        assert_eq!(body, r#"{"user_id": "200"}"#);
    }

    #[test]
    fn deserialize_xml() {
        let body = PayloadValue::Raw {
            format: Some(PayloadRawFormat::Xml),
            body: String::from("<result>hello world</result>"),
        };
        let parsed = RequestBody::from(body);
        assert_eq!(parsed.body_type(), RequestBodyType::Raw);
        let body = parsed.raw().unwrap();
        assert_eq!(body.payload_type(), RequestBodyRawType::Xml);
        assert_eq!(body.payload(), "<result>hello world</result>");
    }

    #[test]
    fn serialize_xml() {
        let body = RequestBody::new(
            RequestBodyType::Raw,
            Some(RequestBodyRaw::new(
                RequestBodyRawType::Xml,
                "<message>hello world</message>",
            )),
        );

        let parsed = PayloadValue::from(body);
        let PayloadValue::Raw { format, body } = parsed else {
            panic!("Not the expected type");
        };
        assert_eq!(format.unwrap(), PayloadRawFormat::Xml);
        assert_eq!(body, "<message>hello world</message>");
    }
}
