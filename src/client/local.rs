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
use thiserror::Error;

use crate::{
    entities::{EndpointData, RequestMethod},
    error::CarteroError,
};

#[derive(Default, Debug, Clone)]
pub struct BoundRequest {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl TryFrom<EndpointData> for BoundRequest {
    type Error = CarteroError;

    fn try_from(value: EndpointData) -> Result<Self, Self::Error> {
        let processor = value.template_processor();

        let url = processor.render(&value.url)?;
        let method = value.method.clone();
        let headers: Result<HashMap<String, String>, CarteroError> = value
            .process_headers()
            .iter()
            .map(|(k, v)| {
                let header_name = processor.render(k)?;
                let header_value = processor.render(v)?;
                Ok((header_name, header_value))
            })
            .collect();
        let headers = headers?;
        let body = match &value.body {
            Some(content) => {
                let string = String::from_utf8_lossy(content);
                let rendered = processor.render(string)?;
                Some(Vec::from(rendered))
            }
            None => None,
        };

        Ok(Self {
            url,
            method,
            headers,
            body,
        })
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Illegal HTTP verb")]
    InvalidHttpVerb,

    #[error("Invalid headers state")]
    InvalidHeaders,

    #[error("Illegal header")]
    InvalidHeaderName(#[from] InvalidHeaderName),

    #[error("Illegal header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("Request error")]
    NetworkError(#[from] isahc::error::Error),

    #[error("HTTP error")]
    HttpError(#[from] isahc::http::Error),

    #[error("Unknown I/O error")]
    IOError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use crate::entities::KeyValueTable;

    use super::*;

    #[test]
    pub fn test_bind_of_parameters() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Get;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_ID", "123412341234").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = Some(Vec::from(b"<client_id>{{ CLIENT_ID }}</client_id>"));
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let bound = BoundRequest::try_from(endpoint).unwrap();

        assert_eq!(bound.url, "https://api.example.com/v1/books");
        assert_eq!(bound.headers["Authorization"], "Bearer 789078907890");
        assert_eq!(
            bound.body,
            Some(Vec::from(b"<client_id>123412341234</client_id>"))
        );
    }

    #[test]
    #[should_panic]
    pub fn test_panics_if_wrong_variable() {
        // Build a request.
        let url = "https://{{API_ROOT}}/v1/books".into();
        let method = RequestMethod::Get;
        let headers = vec![
            ("X-Client-Id", "{{CLIENT_ID}}").into(),
            ("Authorization", "Bearer {{CLIENT_SECRET}}").into(),
            ("Accept", "application/html").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let variables = vec![
            ("API_ROOT", "api.example.com").into(),
            ("CLIENT_SECRET", "789078907890").into(),
        ];
        let variables = KeyValueTable::new(&variables);
        let body = Some(Vec::new());
        let endpoint = EndpointData {
            url,
            method,
            headers,
            variables,
            body,
        };

        // Bind the request.
        let _ = BoundRequest::try_from(endpoint).unwrap();
    }
}
