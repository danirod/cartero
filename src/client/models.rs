use std::collections::HashMap;

use super::RequestError;

#[derive(Debug, Clone, PartialEq)]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: u16,
    pub duration: u64, // miliseconds
    pub size: u64,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn body_as_str(&self) -> String {
        if let Ok(s) = String::from_utf8(self.body.clone()) {
            return s;
        }
        // Fallback to Latin1 in case the contents are not UTF-8
        // TODO: This is not acceptable.
        self.body.iter().map(|&c| c as char).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn new(
        url: String,
        method: RequestMethod,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Self {
        Self {
            url,
            method,
            headers,
            body,
        }
    }
}

impl TryFrom<&str> for RequestMethod {
    type Error = RequestError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "get" => Ok(RequestMethod::Get),
            "post" => Ok(RequestMethod::Post),
            "put" => Ok(RequestMethod::Put),
            "patch" => Ok(RequestMethod::Patch),
            "delete" => Ok(RequestMethod::Delete),
            "options" => Ok(RequestMethod::Options),
            "head" => Ok(RequestMethod::Head),
            _ => Err(RequestError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_convert_str_to_method() {
        assert!(RequestMethod::try_from("GET").is_ok_and(|x| x == RequestMethod::Get));
        assert!(RequestMethod::try_from("post").is_ok_and(|x| x == RequestMethod::Post));
        assert!(RequestMethod::try_from("Patch").is_ok_and(|x| x == RequestMethod::Patch));
        assert!(RequestMethod::try_from("Juan").is_err());
    }
}
