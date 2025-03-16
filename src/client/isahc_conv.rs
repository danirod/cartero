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

use crate::{
    app::CarteroApplication,
    entities::{RequestMethod, ResponseData},
    error::{RequestBuildError, RequestError},
};

use super::BoundRequest;
use futures_lite::io::AsyncReadExt;
use gtk::prelude::SettingsExt;
use isahc::{
    config::{Configurable, SslOption},
    http::{HeaderName, HeaderValue},
    AsyncBody, Body,
};
use std::{
    io::Read,
    time::{Duration, Instant},
};
use url::Url;

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
            RequestMethod::Trace => isahc::http::Method::TRACE,
        }
    }
}

pub fn build_request(req: &BoundRequest) -> Result<isahc::Request<Vec<u8>>, RequestBuildError> {
    let url = Url::parse(&req.url).map_err(|pe| RequestBuildError::InvalidUrl(pe))?;
    let mut builder = isahc::Request::builder()
        .uri(url.as_str())
        .method(&req.method);

    let app = CarteroApplication::default();
    let settings = app.settings();

    if settings.boolean("validate-tls") {
        builder = builder.ssl_options(SslOption::NONE);
    } else {
        builder = builder.ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS);
    }

    if settings.boolean("follow-redirects") {
        let count = settings.uint("maximum-redirects");
        builder = builder.redirect_policy(isahc::config::RedirectPolicy::Limit(count));
    } else {
        builder = builder.redirect_policy(isahc::config::RedirectPolicy::None);
    }

    let timeout = settings.double("request-timeout");
    builder = builder.timeout(Duration::from_secs_f64(timeout));

    let headers = builder.headers_mut().unwrap();
    for (h, v) in &req.headers {
        let key = HeaderName::try_from(h)
            .map_err(|_| RequestBuildError::InvalidHeaderName(h.to_string()))?;
        let value = HeaderValue::try_from(v)
            .map_err(|_| RequestBuildError::InvalidHeaderValue(h.to_string()))?;

        /*
         * Double check that it's actually a valid value. It might be broken and it's only
         * being reported via an expect() and when it's too late to catch the panic:
         * https://docs.rs/crate/isahc/1.7.2/source/src/parsing.rs#60-62
         */
        value
            .to_str()
            .map_err(|_| RequestBuildError::InvalidHeaderValue(h.to_string()))?;

        headers.insert(key, value);
    }
    let body = req.body.clone().unwrap_or_default();
    builder
        .body(body)
        .map_err(|_| RequestBuildError::InvalidBodyEncoding)
}

impl TryFrom<&mut isahc::Response<Body>> for ResponseData {
    type Error = RequestError;

    fn try_from(value: &mut isahc::Response<Body>) -> Result<Self, Self::Error> {
        let status_code = value.status().as_u16() as u32;
        let headers = value
            .headers()
            .iter()
            .map(|(k, v)| {
                let header_name = k.to_string();
                let header_value = String::from(v.to_str().unwrap());
                (header_name, header_value).into()
            })
            .collect();
        let body = {
            let mut buffer = Vec::new();
            let body = value.body_mut();
            body.read_to_end(&mut buffer)
                .map_err(|e| RequestError::IOError(e))?;
            buffer
        };
        Ok(ResponseData {
            duration: 0,
            size: 0,
            status_code,
            headers,
            body,
        })
    }
}

pub async fn extract_isahc_response(
    value: &mut isahc::Response<AsyncBody>,
    start: &Instant,
) -> Result<ResponseData, RequestError> {
    let status_code: u32 = value.status().as_u16() as u32;
    let headers = value
        .headers()
        .iter()
        .map(|(k, v)| {
            let header_name = k.to_string();
            let header_value = String::from(v.to_str().unwrap());
            (header_name, header_value).into()
        })
        .collect();
    let body = {
        let mut buffer = Vec::new();
        let body = value.body_mut();
        body.read_to_end(&mut buffer)
            .await
            .map_err(|e| RequestError::IOError(e))?;
        buffer
    };
    let duration = start.elapsed();
    Ok(ResponseData {
        duration: duration.as_millis(),
        size: body.len(),
        status_code,
        headers,
        body,
    })
}
