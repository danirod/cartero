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

use super::{Request, RequestError, RequestMethod, Response};
use isahc::{
    http::{HeaderName, HeaderValue},
    Body,
};
use std::{io::Read, str::FromStr};

impl From<&RequestMethod> for isahc::http::Method {
    fn from(value: &RequestMethod) -> Self {
        match value {
            RequestMethod::Head => isahc::http::Method::HEAD,
            RequestMethod::Get => isahc::http::Method::GET,
            RequestMethod::Post => isahc::http::Method::POST,
            RequestMethod::Put => isahc::http::Method::PUT,
            RequestMethod::Patch => isahc::http::Method::PATCH,
            RequestMethod::Options => isahc::http::Method::OPTIONS,
            RequestMethod::Delete => isahc::http::Method::DELETE,
        }
    }
}

impl TryFrom<Request> for isahc::Request<Vec<u8>> {
    type Error = RequestError;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        let mut builder = isahc::Request::builder().uri(&req.url).method(&req.method);
        let Some(headers) = builder.headers_mut() else {
            return Err(RequestError);
        };
        for (h, v) in &req.headers {
            let key = HeaderName::from_str(h)?;
            let value = HeaderValue::from_str(v)?;
            headers.insert(key, value);
        }
        let req = builder.body(req.body.clone())?;
        Ok(req)
    }
}

impl TryFrom<&mut isahc::Response<Body>> for Response {
    type Error = RequestError;

    fn try_from(value: &mut isahc::Response<Body>) -> Result<Self, Self::Error> {
        let status_code: u16 = value.status().as_u16();
        let headers = value
            .headers()
            .iter()
            .map(|(k, v)| {
                let header_name = k.to_string();
                let header_value = String::from(v.to_str().unwrap());
                (header_name, header_value)
            })
            .collect();
        let body = {
            let mut buffer = Vec::new();
            let body = value.body_mut();
            body.read_to_end(&mut buffer)?;
            buffer
        };
        Ok(Response {
            duration: 0,
            size: 0,
            status_code,
            headers,
            body,
        })
    }
}
