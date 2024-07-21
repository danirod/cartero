use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use serde::{Deserialize, Serialize};

use crate::client::RequestError;
use crate::entities::{
    EndpointData, KeyValue, KeyValueTable, RawEncoding, RequestMethod, RequestPayload,
};
use crate::error::CarteroError;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct KeyValueDetail {
    value: String,
    active: bool,
    secret: bool,
}

impl Default for KeyValueDetail {
    fn default() -> Self {
        Self {
            value: String::default(),
            active: true,
            secret: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum KeyValuedValue {
    Simple(String),
    Complex(KeyValueDetail),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum KeyValuedValueContainer {
    Unique(KeyValuedValue),
    Multiple(Vec<KeyValuedValue>),
}

impl Default for KeyValuedValue {
    fn default() -> Self {
        Self::Simple(String::default())
    }
}

impl From<KeyValuedValue> for KeyValue {
    fn from(value: KeyValuedValue) -> KeyValue {
        match value {
            KeyValuedValue::Simple(str) => KeyValue {
                name: String::default(),
                value: str.clone(),
                active: true,
                secret: false,
            },
            KeyValuedValue::Complex(kd) => KeyValue {
                name: String::default(),
                value: kd.value.clone(),
                active: kd.active,
                secret: kd.secret,
            },
        }
    }
}

impl From<KeyValue> for KeyValuedValue {
    fn from(value: KeyValue) -> Self {
        let def = KeyValueDetail::default();
        if value.active == def.active && value.secret == def.secret {
            Self::Simple(value.value)
        } else {
            Self::Complex(KeyValueDetail {
                active: value.active,
                secret: value.secret,
                value: value.value,
            })
        }
    }
}

impl From<Vec<KeyValue>> for KeyValuedValueContainer {
    fn from(value: Vec<KeyValue>) -> Self {
        if value.len() == 1 {
            KeyValuedValueContainer::Unique(value[0].clone().into())
        } else {
            let multiple: Vec<KeyValuedValue> =
                value.into_iter().map(KeyValuedValue::from).collect();
            KeyValuedValueContainer::Multiple(multiple)
        }
    }
}

fn extract_kv_entry(value: KeyValuedValue, key: &str) -> KeyValue {
    let mut value = KeyValue::from(value);
    value.name = key.into();
    value
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct KeyValuedFileTable(HashMap<String, KeyValuedValueContainer>);

impl From<KeyValuedFileTable> for KeyValueTable {
    fn from(value: KeyValuedFileTable) -> Self {
        let mut vector: Vec<KeyValue> = value
            .0
            .into_iter()
            .flat_map(|(header, values)| match values {
                KeyValuedValueContainer::Unique(x) => vec![extract_kv_entry(x, &header)],
                KeyValuedValueContainer::Multiple(mult) => mult
                    .into_iter()
                    .map(|v| extract_kv_entry(v, &header))
                    .collect(),
            })
            .collect();
        vector.sort();
        KeyValueTable::new(&vector)
    }
}

impl From<KeyValueTable> for KeyValuedFileTable {
    fn from(value: KeyValueTable) -> Self {
        let group = value.group_by();
        let inner = group
            .into_iter()
            .map(|(key, vector)| (key, vector.into()))
            .collect();
        Self(inner)
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub enum FilePayloadRawFormat {
    #[default]
    #[serde(rename = "octet-stream")]
    OctetStream,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "xml")]
    Xml,
}

impl From<RawEncoding> for FilePayloadRawFormat {
    fn from(value: RawEncoding) -> Self {
        match value {
            RawEncoding::Json => Self::Json,
            RawEncoding::OctetStream => Self::OctetStream,
            RawEncoding::Xml => Self::Xml,
        }
    }
}

impl From<FilePayloadRawFormat> for RawEncoding {
    fn from(value: FilePayloadRawFormat) -> Self {
        match value {
            FilePayloadRawFormat::Json => Self::Json,
            FilePayloadRawFormat::OctetStream => Self::OctetStream,
            FilePayloadRawFormat::Xml => Self::Xml,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FilePayload {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "urlencoded")]
    UrlEncoded {
        variables: Option<KeyValuedFileTable>,
    },
    #[serde(rename = "multipart")]
    Multipart {
        variables: Option<KeyValuedFileTable>,
    },
    #[serde(rename = "raw")]
    Raw {
        format: Option<FilePayloadRawFormat>,
        body: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Body {
    ClassicRaw(String),
    Structured(FilePayload),
}

impl From<RequestPayload> for FilePayload {
    fn from(value: RequestPayload) -> Self {
        match value {
            RequestPayload::None => Self::None,
            RequestPayload::Urlencoded(payload) => Self::UrlEncoded {
                variables: Some(payload.into()),
            },
            RequestPayload::Multipart { params } => Self::Multipart {
                variables: Some(params.into()),
            },
            RequestPayload::Raw { encoding, content } => Self::Raw {
                format: Some(encoding.into()),
                body: String::from_utf8_lossy(&content.clone()).to_string(),
            },
        }
    }
}

impl From<FilePayload> for RequestPayload {
    fn from(value: FilePayload) -> Self {
        match value {
            FilePayload::None => Self::None,
            FilePayload::Multipart { variables } => Self::Multipart {
                params: variables.unwrap_or_default().into(),
            },
            FilePayload::UrlEncoded { variables } => {
                Self::Urlencoded(variables.unwrap_or_default().into())
            }
            FilePayload::Raw { format, body } => Self::Raw {
                encoding: format.unwrap_or_default().into(),
                content: Vec::from(body.clone()),
            },
        }
    }
}

impl From<RequestPayload> for Body {
    fn from(value: RequestPayload) -> Self {
        Self::Structured(value.into())
    }
}

impl From<Body> for RequestPayload {
    fn from(value: Body) -> Self {
        match value {
            Body::ClassicRaw(payload) => Self::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(payload.clone().as_str()),
            },
            Body::Structured(payload) => payload.into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct RequestFile {
    version: usize,
    url: String,
    method: String,
    body: Option<Body>,
    headers: Option<KeyValuedFileTable>,
    variables: Option<KeyValuedFileTable>,
}

impl TryFrom<RequestFile> for EndpointData {
    type Error = CarteroError;

    fn try_from(value: RequestFile) -> Result<EndpointData, Self::Error> {
        if value.version != 1 {
            return Err(CarteroError::OutdatedSchema);
        }
        let Ok(method) = RequestMethod::try_from(value.method.as_str()) else {
            return Err(RequestError::InvalidHttpVerb.into());
        };
        let body = value.body.map(RequestPayload::from).unwrap_or_default();
        let headers = value.headers.unwrap_or_default().into();
        let variables = value.variables.unwrap_or_default().into();

        let request = EndpointData {
            url: value.url.clone(),
            method,
            body,
            variables,
            headers,
        };
        Ok(request)
    }
}

impl From<EndpointData> for RequestFile {
    fn from(value: EndpointData) -> RequestFile {
        let method: &str = value.method.into();
        let body = match value.body {
            RequestPayload::None => None,
            otherwise => Some(otherwise.into()),
        };
        let headers = value.headers.into();
        let variables = value.variables.into();
        RequestFile {
            version: 1,
            url: value.url.clone(),
            method: method.to_owned(),
            body,
            headers: Some(headers),
            variables: Some(variables),
        }
    }
}

pub fn parse_toml(file: &str) -> Result<EndpointData, CarteroError> {
    let contents = toml::from_str::<RequestFile>(file)?;
    EndpointData::try_from(contents)
}

pub fn store_toml(endpoint: &EndpointData) -> Result<String, CarteroError> {
    let file = RequestFile::from(endpoint.clone());
    toml::to_string(&file).map_err(|e| e.into())
}

pub fn read_file(path: &PathBuf) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn write_file(path: &PathBuf, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    write!(file, "{}", contents)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        entities::{
            EndpointData, KeyValue, KeyValueTable, RawEncoding, RequestMethod, RequestPayload,
        },
        file::KeyValueDetail,
    };

    use super::{KeyValuedFileTable, KeyValuedValueContainer};

    #[test]
    pub fn test_key_valued_file_table_to_key_value_table_sorts_simple() {
        let map = HashMap::from([
            (
                "User-Agent".into(),
                KeyValuedValueContainer::Unique(super::KeyValuedValue::Simple(
                    "Cartero/0.1".into(),
                )),
            ),
            (
                "Host".into(),
                KeyValuedValueContainer::Unique(super::KeyValuedValue::Simple(
                    "www.google.com".into(),
                )),
            ),
        ]);
        let file = KeyValuedFileTable(map);

        let table = KeyValueTable::from(file);

        assert_eq!(table.len(), 2);
        assert_eq!(
            table,
            KeyValueTable::new(&[
                ("Host", "www.google.com").into(),
                ("User-Agent", "Cartero/0.1").into()
            ])
        );
    }

    #[test]
    pub fn test_key_valued_file_table_to_key_value_table_sorts_complex() {
        let map = HashMap::from([
            (
                "User-Agent".into(),
                KeyValuedValueContainer::Unique(super::KeyValuedValue::Complex(KeyValueDetail {
                    value: "Cartero/0.1".into(),
                    active: false,
                    secret: true,
                })),
            ),
            (
                "Host".into(),
                KeyValuedValueContainer::Unique(super::KeyValuedValue::Simple(
                    "www.google.com".into(),
                )),
            ),
        ]);
        let file = KeyValuedFileTable(map);

        let table = KeyValueTable::from(file);

        assert_eq!(table.len(), 2);
        assert_eq!(
            table,
            KeyValueTable::new(&[
                ("Host", "www.google.com").into(),
                KeyValue {
                    name: "User-Agent".into(),
                    value: "Cartero/0.1".into(),
                    active: false,
                    secret: true,
                }
            ])
        );
    }

    #[test]
    pub fn test_key_valued_file_table_to_key_value_table_sorts_multiple() {
        let map = HashMap::from([
            (
                "User-Agent".into(),
                KeyValuedValueContainer::Unique(super::KeyValuedValue::Simple(
                    "Cartero/0.1".into(),
                )),
            ),
            (
                "Accept".into(),
                KeyValuedValueContainer::Multiple(vec![
                    super::KeyValuedValue::Simple("*/*".into()),
                    super::KeyValuedValue::Simple("application/json".into()),
                    super::KeyValuedValue::Simple("application/ld+json".into()),
                ]),
            ),
            (
                "Host".into(),
                KeyValuedValueContainer::Unique(super::KeyValuedValue::Simple(
                    "www.google.com".into(),
                )),
            ),
        ]);
        let file = KeyValuedFileTable(map);

        let table = KeyValueTable::from(file);

        assert_eq!(table.len(), 5);
        assert_eq!(
            table,
            KeyValueTable::new(&vec![
                ("Accept", "*/*").into(),
                ("Accept", "application/json").into(),
                ("Accept", "application/ld+json").into(),
                ("Host", "www.google.com").into(),
                ("User-Agent", "Cartero/0.1").into(),
            ])
        );
    }

    #[test]
    pub fn test_can_deserialize_classic() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = 'text/html'
Accept-Encoding = 'gzip'
";
        let endpoint = super::parse_toml(toml).unwrap();
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 2);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&[
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: true,
                    secret: false
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                }
            ]),
        );
    }

    #[test]
    pub fn test_can_deserialize_complex_headers() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = { value = 'text/html', secret = true, active = false }
Accept-Encoding = 'gzip'
";
        let endpoint = super::parse_toml(toml).unwrap();
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 2);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&[
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: false,
                    secret: true,
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                }
            ]),
        );
    }

    #[test]
    pub fn test_can_deserialize_header_arrays() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = ['application/json', 'text/html']
Accept-Encoding = 'gzip'
";
        let endpoint = super::parse_toml(toml).unwrap();
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 3);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&[
                KeyValue {
                    name: "Accept".into(),
                    value: "application/json".into(),
                    active: true,
                    secret: false,
                },
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: true,
                    secret: false,
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                }
            ]),
        );
    }

    #[test]
    pub fn test_deserialize_complex_header_arrays() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = [
    { value = 'application/json', active = false, secret = false },
    { value = 'text/html', active = false, secret = false },
]
X-Client-Id = [
    { value = '123412341234', active = true, secret = true },
    { value = '{{CLIENT_ID}}', active = false, secret = false },
]
Accept-Encoding = 'gzip'
";
        let endpoint = super::parse_toml(toml).unwrap();
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 5);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&vec![
                KeyValue {
                    name: "Accept".into(),
                    value: "application/json".into(),
                    active: false,
                    secret: false,
                },
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: false,
                    secret: false,
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                },
                KeyValue {
                    name: "X-Client-Id".into(),
                    value: "123412341234".into(),
                    active: true,
                    secret: true
                },
                KeyValue {
                    name: "X-Client-Id".into(),
                    value: "{{CLIENT_ID}}".into(),
                    active: false,
                    secret: false
                },
            ]),
        );
    }

    #[test]
    pub fn test_deserialization_error() {
        let toml = "
version = 0
url = 'https://www.google.com'
method = 'GET'
body = 'hello'
";
        assert!(super::parse_toml(toml).is_err());
    }

    #[test]
    pub fn test_method_error() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'THROW'
";
        assert!(super::parse_toml(toml).is_err());
    }

    #[test]
    pub fn test_empty_url() {
        let toml = "
version = 1
method = 'POST'
body = 'hello'

[headers]
Accept = 'text/html'
";
        assert!(super::parse_toml(toml).is_err());
    }

    #[test]
    pub fn test_empty_method() {
        let toml = "
version = 1
url = 'https://www.google.com'
body = 'hello'

[headers]
Accept = 'text/html'
";
        assert!(super::parse_toml(toml).is_err());
    }

    #[test]
    pub fn test_empty_body() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'

[headers]
Accept = 'text/html'
";
        let endpoint = super::parse_toml(toml).unwrap();
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(endpoint.body, RequestPayload::None);
    }

    #[test]
    pub fn test_multiple_headers_serialization() {
        let headers = vec![
            ("Host", "google.com").into(),
            ("User-Agent", "Cartero").into(),
            ("User-Agent", "Cartero/0.1").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = RequestPayload::None;
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables: KeyValueTable::default(),
            body,
        };

        let content = super::store_toml(&r).unwrap();
        let content = content.as_str();
        assert!(content.contains("url = \"https://www.google.com\""));
        assert!(content.contains("Host = \"google.com\""));
        assert!(content.contains("User-Agent = ["));
    }

    #[test]
    pub fn test_multiple_headers_serialization_with_meta() {
        let headers = vec![
            ("Host", "google.com").into(),
            ("User-Agent", "Cartero").into(),
            KeyValue {
                name: "User-Agent".into(),
                value: "Cartero/devel".into(),
                active: false,
                secret: false,
            },
            ("User-Agent", "Cartero/0.1").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = RequestPayload::None;
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables: KeyValueTable::default(),
            body,
        };

        let content = super::store_toml(&r).unwrap();
        let content = content.as_str();
        assert!(content.contains("url = \"https://www.google.com\""));
        assert!(content.contains("Host = \"google.com\""));
        assert!(content.contains("User-Agent = ["));
        assert!(content.contains("active = false"));
    }

    #[test]
    pub fn test_empty_headers() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'POST'
body = 'hello'
";
        let endpoint = super::parse_toml(toml).unwrap();
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Post);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                content: Vec::from(b"hello"),
                encoding: RawEncoding::OctetStream,
            }
        );
        assert_eq!(endpoint.headers.len(), 0);
    }

    #[test]
    pub fn test_serialize_correctly() {
        let headers = vec![
            ("User-Agent", "Cartero").into(),
            ("Host", "google.com").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = RequestPayload::Raw {
            content: Vec::from(b"Hello"),
            encoding: RawEncoding::OctetStream,
        };
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables: KeyValueTable::default(),
            body,
        };

        let content = super::store_toml(&r).unwrap();
        assert!(content
            .as_str()
            .contains("url = \"https://www.google.com\""));
        assert!(content.as_str().contains("method = \"POST\""));
        assert!(content.as_str().contains("body = \"Hello\""));
        assert!(content.as_str().contains("User-Agent = \"Cartero\""));
    }

    #[test]
    pub fn test_serializes_complex_example() {
        // One thing important to test: since this is eventually a hashmap, the result
        // will be sorted by key name, but the order of the elements must match the
        // original order.
        let headers = KeyValueTable::new(&vec![
            KeyValue {
                name: "X-Client-Id".into(),
                value: "123412341234".into(),
                secret: true,
                active: true,
            },
            ("Host", "google.com").into(),
            ("User-Agent", "Cartero").into(),
            KeyValue {
                name: "X-Client-Id".into(),
                value: "{{CLIENT_ID}}".into(),
                secret: false,
                active: false,
            },
        ]);
        let variables = KeyValueTable::new(&[
            KeyValue {
                name: "CLIENT_SECRET".into(),
                value: "101010".into(),
                secret: true,
                active: true,
            },
            ("CLIENT_ID", "123412341234").into(),
            KeyValue {
                name: "CLIENT_SECRET".into(),
                value: "202020".into(),
                secret: true,
                active: true,
            },
        ]);
        let body = RequestPayload::Raw {
            content: Vec::from(b"Hello"),
            encoding: RawEncoding::OctetStream,
        };
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables,
            body,
        };

        let content = super::store_toml(&r).unwrap();
        let parsed = super::parse_toml(&content).unwrap();
        assert_eq!(r.url, parsed.url);
        assert_eq!(r.method, parsed.method);
        assert_eq!(r.body, parsed.body);

        assert_eq!(
            KeyValueTable::new(&vec![
                r.headers[1].clone(),
                r.headers[2].clone(),
                r.headers[0].clone(),
                r.headers[3].clone(),
            ]),
            parsed.headers
        );
        assert_eq!(
            KeyValueTable::new(&[
                r.variables[1].clone(),
                r.variables[0].clone(),
                r.variables[2].clone()
            ]),
            parsed.variables
        );
    }
}
