// Copyright 2024 the Cartero authors
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

use isahc::http::header::{InvalidHeaderName, InvalidHeaderValue};
use std::collections::HashMap;
use std::error::Error;

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

#[derive(Debug)]
pub struct RequestError;

impl Error for RequestError {}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request Error")
    }
}

impl From<InvalidHeaderName> for RequestError {
    fn from(_: InvalidHeaderName) -> Self {
        Self
    }
}

impl From<InvalidHeaderValue> for RequestError {
    fn from(_: InvalidHeaderValue) -> Self {
        Self
    }
}

impl From<&str> for RequestError {
    fn from(_: &str) -> Self {
        Self
    }
}

impl From<isahc::http::Error> for RequestError {
    fn from(_: isahc::http::Error) -> Self {
        Self
    }
}

impl From<std::io::Error> for RequestError {
    fn from(_: std::io::Error) -> Self {
        Self
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
