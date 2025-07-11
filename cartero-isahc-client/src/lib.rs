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

#![doc = include_str!("../README.md")]

use futures_lite::io::AsyncReadExt;
use std::time::{Duration, Instant};

use cartero_http::{BoundRequest, RequestEnvironment, RequestError};
use cartero_objects::{Field, FieldTable, Request, RequestMethod, Response};
use isahc::{
    config::{Configurable, RedirectPolicy, SslOption},
    http::{HeaderName, HeaderValue},
    AsyncBody, RequestExt,
};

fn default_user_agent() -> String {
    let cartero_version = env!("CARGO_PKG_VERSION");
    let curl_version = {
        let version = curl::Version::get();
        version.version().to_string()
    };
    format!("Cartero/{cartero_version} (curl/{curl_version})")
}

pub async fn request(
    request: &Request,
    env: &RequestEnvironment,
) -> Result<Response, RequestError> {
    // Craft an isahc request.
    let isahc_request = build_request(request, env)?;

    // Execute the request to get the response.
    let start = Instant::now();
    let mut response = isahc_request
        .send_async()
        .await
        .map_err(|e| RequestError::NetworkError(Box::new(e)))?;
    build_response(request, &mut response, &start).await
}

async fn build_response(
    request: &Request,
    isahc_request: &mut isahc::Response<AsyncBody>,
    start: &Instant,
) -> Result<Response, RequestError> {
    let status_code = isahc_request.status().as_u16();
    let headers = isahc_request.headers().iter().map(|(k, v)| {
        let name = k.to_string();
        let value = String::from(v.to_str().unwrap());
        Field::builder().key(name).value(value).build()
    });
    let headers = FieldTable::from_iter(headers);
    let body = {
        let mut buffer = Vec::new();
        let body = isahc_request.body_mut();
        body.read_to_end(&mut buffer)
            .await
            .map_err(|e| RequestError::IOError(Box::new(e)))?;
        buffer
    };

    let duration = start.elapsed();

    let response = Response::builder(request)
        .status_code(status_code as u32)
        .headers(&headers)
        .size(body.len() as u64)
        .body(&body)
        .duration(duration.as_millis() as u64)
        .build();
    Ok(response)
}

fn build_request(
    request: &Request,
    env: &RequestEnvironment,
) -> Result<isahc::Request<Vec<u8>>, RequestError> {
    // Extract isahc settings from the environment
    let ssl_mode = if env.config.validate_tls {
        SslOption::NONE
    } else {
        SslOption::DANGER_ACCEPT_INVALID_CERTS
    };
    let redirect_policy = if env.config.redirects > 0 {
        RedirectPolicy::Limit(env.config.redirects as u32)
    } else {
        RedirectPolicy::None
    };
    let request_timeout = Duration::from_secs_f64(env.config.timeout);

    // Build the request entity.
    let bound_request = BoundRequest::try_from(request.clone())?;

    let mut builder = isahc::Request::builder()
        .uri(bound_request.url)
        .method(isahc_request_method(&request.method()))
        .ssl_options(ssl_mode)
        .redirect_policy(redirect_policy)
        .timeout(request_timeout);
    let headers = builder.headers_mut().unwrap();
    for (key, value) in &bound_request.headers {
        let header_key = HeaderName::try_from(key)
            .map_err(|_| RequestError::InvalidHeaderName(key.to_string()))?;
        let header_value = HeaderValue::try_from(value)
            .map_err(|_| RequestError::InvalidHeaderValue(key.to_string()))?;

        // Doubule check that the header is actually valid UTF-8. I don't care about
        // the result, I just want to check if it fails to convert or not.
        // isahc will do this later, but instead of returning an error they just
        // expect() and panic(). Better catch the error than having the app killed
        // and I don't want to use panic handlers for this.
        header_value
            .to_str()
            .map_err(|_| RequestError::InvalidHeaderValue(key.to_string()))?;

        headers.insert(header_key, header_value);
    }

    // Add user agent if not added yet.
    if !headers.contains_key("user-agent") {
        headers.insert("user-agent", default_user_agent().try_into().unwrap());
    }

    let body = bound_request.body.unwrap_or_default();
    builder.body(body).map_err(|_| RequestError::EncodingError)
}

fn isahc_request_method(cartero_method: &RequestMethod) -> isahc::http::Method {
    match cartero_method {
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
