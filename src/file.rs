use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use serde::{Deserialize, Serialize};

use crate::client::{Request, RequestMethod};

#[derive(Deserialize, Serialize)]
struct RequestFile {
    version: usize,
    url: String,
    method: String,
    body: Option<String>,
    headers: Option<HashMap<String, String>>,
}

impl TryFrom<RequestFile> for Request {
    type Error = &'static str;

    fn try_from(value: RequestFile) -> Result<Request, Self::Error> {
        if value.version != 1 {
            return Err("Unsupported version, please upgrade the software");
        }
        let Ok(method) = RequestMethod::try_from(value.method.as_str()) else {
            return Err("Invalid method");
        };
        let body = match value.body {
            Some(b) => Vec::from(b.as_str()),
            None => Vec::new(),
        };
        let headers: HashMap<String, String> = match value.headers {
            Some(hdrs) => hdrs.clone(),
            None => HashMap::new(),
        };
        let request = Request {
            url: value.url.clone(),
            method,
            body,
            headers,
        };
        Ok(request)
    }
}

impl From<Request> for RequestFile {
    fn from(value: Request) -> RequestFile {
        let method: &str = value.method.into();
        let body = if value.body.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&value.body.clone()).to_string())
        };
        RequestFile {
            version: 1,
            url: value.url.clone(),
            method: method.to_owned(),
            body,
            headers: Some(value.headers.clone()),
        }
    }
}

pub fn parse_toml(file: &str) -> Result<Request, ()> {
    let Ok(contents) = toml::from_str::<RequestFile>(file) else {
        return Err(());
    };
    Request::try_from(contents).map_err(|_| ())
}

pub fn store_toml(req: &Request) -> Result<String, ()> {
    let file = RequestFile::from(req.clone());
    toml::to_string(&file).map_err(|_| ())
}

pub fn write_file(path: &PathBuf, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    write!(file, "{}", contents)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::client::{Request, RequestMethod};

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
        let doc = super::parse_toml(toml).unwrap();
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
        let content = super::parse_toml(toml).unwrap();
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
        let content = super::parse_toml(toml).unwrap();
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

        let content = super::store_toml(&r).unwrap();
        let parsed = super::parse_toml(&content).unwrap();
        assert_eq!(r.url, parsed.url);
        assert_eq!(r.method, parsed.method);
        assert_eq!(r.body, parsed.body);
        assert_eq!(r.headers, parsed.headers);
    }
}
