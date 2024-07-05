use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use serde::{Deserialize, Serialize};

use crate::client::{KeyValueData, Request, RequestError, RequestMethod};
use crate::error::CarteroError;
use crate::objects::Endpoint;

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

impl From<KeyValuedValue> for KeyValueData {
    fn from(value: KeyValuedValue) -> KeyValueData {
        match value {
            KeyValuedValue::Simple(str) => KeyValueData {
                value: str.clone(),
                active: true,
                secret: false,
            },
            KeyValuedValue::Complex(kd) => KeyValueData {
                value: kd.value.clone(),
                active: kd.active,
                secret: kd.secret,
            },
        }
    }
}

impl From<KeyValueData> for KeyValuedValue {
    fn from(value: KeyValueData) -> Self {
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

impl TryFrom<RequestFile> for Request {
    type Error = CarteroError;

    fn try_from(value: RequestFile) -> Result<Request, Self::Error> {
        if value.version != 1 {
            return Err(CarteroError::OutdatedSchema);
        }
        let Ok(method) = RequestMethod::try_from(value.method.as_str()) else {
            return Err(RequestError::InvalidHttpVerb.into());
        };
        let body = match value.body {
            Some(b) => Vec::from(b.as_str()),
            None => Vec::new(),
        };
        let headers = value
            .headers
            .unwrap_or_default()
            .into_iter()
            .map(|(k, data)| (k.clone(), KeyValueData::from(data)))
            .collect();
        let request = Request {
            url: value.url.clone(),
            method,
            body,
            headers,
        };
        Ok(request)
    }
}

impl From<Endpoint> for RequestFile {
    fn from(value: Endpoint) -> RequestFile {
        let Endpoint(request, variables) = value;

        let method: &str = request.method.into();
        let body = if request.body.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&request.body.clone()).to_string())
        };

        let headers = request
            .headers
            .into_iter()
            .map(|(k, data)| (k.clone(), KeyValuedValue::from(data)))
            .collect();
        let variables = variables
            .into_iter()
            .map(|(k, data)| (k.clone(), KeyValuedValue::from(data)))
            .collect();

        RequestFile {
            version: 1,
            url: request.url.clone(),
            method: method.to_owned(),
            body,
            headers: Some(headers),
            variables: Some(variables),
        }
    }
}

pub fn parse_toml(file: &str) -> Result<Endpoint, CarteroError> {
    let contents = toml::from_str::<RequestFile>(file)?;
    let variables = contents.variables.clone().unwrap_or(HashMap::new());
    let request = Request::try_from(contents)?;
    let variables = variables
        .into_iter()
        .map(|(k, d)| (k.clone(), KeyValueData::from(d)))
        .collect();
    Ok(Endpoint(request, variables))
}

pub fn store_toml(endpoint: Endpoint) -> Result<String, CarteroError> {
    let file = RequestFile::from(endpoint);
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
        client::{Request, RequestMethod},
        objects::Endpoint,
    };

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
        let Endpoint(doc, _) = endpoint;
        assert_eq!(doc.url, "https://www.google.com");
        assert_eq!(doc.method, RequestMethod::Get);
        assert_eq!(doc.body, vec![0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        assert_eq!(doc.headers.len(), 2);
        assert_eq!(doc.headers["Accept"], "text/html");
        assert_eq!(doc.headers["Accept-Encoding"], "gzip");
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
        let Endpoint(content, _) = endpoint;
        assert_eq!(content.url, "https://www.google.com");
        assert_eq!(content.method, RequestMethod::Get);
        assert_eq!(content.body.len(), 0);
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
        let Endpoint(content, _) = endpoint;
        assert_eq!(content.url, "https://www.google.com");
        assert_eq!(content.method, RequestMethod::Post);
        assert_eq!(content.body.len(), 5);
        assert_eq!(content.headers.len(), 0);
    }

    #[test]
    pub fn test_serialize_correctly() {
        let mut headers = HashMap::default();
        headers.insert("User-Agent".to_string(), "Cartero".to_string());
        headers.insert("Host".to_string(), "google.com".to_string());
        let body = Vec::from("Hello");
        let r = Request::new(
            "https://www.google.com".to_string(),
            RequestMethod::Post,
            headers,
            body,
        );

        let content = super::store_toml(Endpoint(r, HashMap::default())).unwrap();
        assert!(content
            .as_str()
            .contains("url = \"https://www.google.com\""));
        assert!(content.as_str().contains("method = \"POST\""));
        assert!(content.as_str().contains("body = \"Hello\""));
        assert!(content.as_str().contains("User-Agent = \"Cartero\""));
    }

    #[test]
    pub fn test_just_for_fun() {
        let mut headers = HashMap::default();
        headers.insert("User-Agent".to_string(), "Cartero".to_string());
        headers.insert("Host".to_string(), "google.com".to_string());
        let body = Vec::from("Hello");
        let r = Request::new(
            "https://www.google.com".to_string(),
            RequestMethod::Post,
            headers,
            body,
        );

        let content = super::store_toml(Endpoint(r.clone(), HashMap::default())).unwrap();
        let parsed = super::parse_toml(&content).unwrap();
        let parsed = parsed.0;
        assert_eq!(r.url, parsed.url);
        assert_eq!(r.method, parsed.method);
        assert_eq!(r.body, parsed.body);
        assert_eq!(r.headers, parsed.headers);
    }
}
