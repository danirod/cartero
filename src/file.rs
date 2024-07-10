use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use serde::{Deserialize, Serialize};

use crate::client::RequestError;
use crate::entities::{EndpointData, KeyValue, KeyValueTable, RequestMethod};
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

#[derive(Deserialize, Serialize)]
struct RequestFile {
    version: usize,
    url: String,
    method: String,
    body: Option<String>,
    headers: Option<HashMap<String, KeyValuedValue>>,
    variables: Option<HashMap<String, KeyValuedValue>>,
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
        let body = value.body.map(|b| Vec::from(b.as_str()));
        let p_headers: KeyValueTable = value
            .headers
            .unwrap_or_default()
            .into_iter()
            .map(|(k, data)| {
                let mut kv = KeyValue::from(data);
                kv.name = k;
                kv
            })
            .collect();
        let p_variables: KeyValueTable = value
            .variables
            .unwrap_or_default()
            .into_iter()
            .map(|(k, data)| {
                let mut kv = KeyValue::from(data);
                kv.name = k;
                kv
            })
            .collect();

        let mut headers = p_headers.clone();
        headers.sort();
        let mut variables = p_variables.clone();
        variables.sort();

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
        let body = value
            .body
            .map(|body| String::from_utf8_lossy(&body).to_string());

        let mut ep_headers = value.headers.clone();
        ep_headers.sort();
        let mut ep_variables = value.variables.clone();
        ep_variables.sort();

        let headers = ep_headers
            .iter()
            .map(|kv| (kv.name.clone(), KeyValuedValue::from(kv.clone())))
            .collect();
        let variables = ep_variables
            .iter()
            .map(|kv| (kv.name.clone(), KeyValuedValue::from(kv.clone())))
            .collect();

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
    use crate::entities::{EndpointData, KeyValue, KeyValueTable, RequestMethod};

    #[test]
    pub fn test_can_deserialize() {
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
        assert_eq!(endpoint.body, Some(Vec::from(b"hello")));
        assert_eq!(endpoint.headers.len(), 2);
        assert_eq!(
            endpoint.headers[0],
            KeyValue {
                name: "Accept".into(),
                value: "text/html".into(),
                active: true,
                secret: false
            }
        );
        assert_eq!(
            endpoint.headers[1],
            KeyValue {
                name: "Accept-Encoding".into(),
                value: "gzip".into(),
                active: true,
                secret: false
            }
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
        assert!(endpoint.body.is_none());
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
        assert_eq!(endpoint.body.unwrap().len(), 5);
        assert_eq!(endpoint.headers.len(), 0);
    }

    #[test]
    pub fn test_serialize_correctly() {
        let headers = vec![
            ("User-Agent", "Cartero").into(),
            ("Host", "google.com").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = Some(Vec::from("Hello"));
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
    pub fn test_just_for_fun() {
        let headers = vec![
            ("Host", "google.com").into(),
            ("User-Agent", "Cartero").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = Some(Vec::from("Hello"));
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables: KeyValueTable::default(),
            body,
        };

        let content = super::store_toml(&r).unwrap();
        let parsed = super::parse_toml(&content).unwrap();
        assert_eq!(r.url, parsed.url);
        assert_eq!(r.method, parsed.method);
        assert_eq!(r.body, parsed.body);
        assert_eq!(r.headers, parsed.headers);
    }
}
