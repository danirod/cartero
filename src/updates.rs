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

use std::time::Instant;

use isahc::RequestExt;
use serde::Deserialize;

use crate::{
    client::BoundRequest,
    entities::{EndpointData, KeyValueTable, RequestMethod, RequestPayload},
};

/// A little shim so that I can extract the information needed from the endpoint.
#[derive(Deserialize)]
pub struct GitHubApiResponse {
    tag_name: String,
}

impl GitHubApiResponse {
    fn from_api_response(payload: &str) -> Option<Self> {
        serde_json::from_str(&payload).ok()
    }

    pub fn get_latest_version(&self) -> String {
        self.tag_name[1..].to_string()
    }

    pub fn needs_update(&self, current_version: &str) -> bool {
        // This function should assume a little that I will always tag my releases as vX.Y.Z
        // and that the day that I change the format, I am completely fucked. Just a little
        // of validation, but please never remove the v from the tag numbers!
        let unstrip = if self.tag_name.starts_with("v") {
            &self.tag_name[1..]
        } else {
            self.tag_name.as_str()
        };
        current_version < unstrip
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_api_response_deserializes() {
        let payload = "{\"tag_name\": \"v0.2.0\"}";
        let response = super::GitHubApiResponse::from_api_response(payload);
        assert!(response.is_some_and(|r| r.get_latest_version() == "0.2.0"));
    }

    #[test]
    pub fn test_api_response_silently_fails() {
        let payload = "{\"error\": \"true\", \"status\": 200}";
        let response = super::GitHubApiResponse::from_api_response(payload);
        assert!(response.is_none());
    }

    #[test]
    pub fn test_api_response_get_latest_version() {
        let gh_response = super::GitHubApiResponse {
            tag_name: "v0.1.5".to_string(),
        };
        assert_eq!("0.1.5", gh_response.get_latest_version());
    }

    #[test]
    pub fn test_api_response_needs_update() {
        let needs_update = vec![
            ("v0.1.5", "0.1.4"),
            ("v0.2.0", "0.1.5"),
            ("v0.3.1", "0.2.6"),
            ("v1.0.0", "0.4.2"),
        ];
        let updated = vec![
            ("v0.1.5", "0.1.5"),
            ("v0.1.5", "0.1.6"),
            ("v0.1.5", "0.2.0"),
            ("v1.0.0", "1.0.0"),
        ];
        for (latest, current) in needs_update {
            let response = super::GitHubApiResponse {
                tag_name: latest.to_string(),
            };
            assert!(response.needs_update(current));
        }
        for (latest, current) in updated {
            let response = super::GitHubApiResponse {
                tag_name: latest.to_string(),
            };
            assert!(!response.needs_update(current));
        }
    }
}

// const API_LATEST_URL: &'static str = "https://api.github.com/repos/danirod/cartero/releases/latest";
const API_LATEST_URL: &'static str = "http://localhost:8000/latest";

fn create_api_check_endpoint() -> EndpointData {
    let headers = vec![("Accept", "application/json").into()];
    EndpointData {
        url: API_LATEST_URL.into(),
        method: RequestMethod::Get,
        parameters: KeyValueTable::default(),
        headers: KeyValueTable::new(&headers),
        body: RequestPayload::None,
        variables: KeyValueTable::default(),
    }
}

// Silently discards any error.
// TODO: at least could log it...
async fn fetch_response(payload: EndpointData) -> Option<String> {
    let endpoint = BoundRequest::try_from(payload).ok()?;
    let request = crate::client::build_request(&endpoint).ok()?;
    let mut response = request.send_async().await.ok()?;
    let response = crate::client::extract_isahc_response(&mut response, &Instant::now())
        .await
        .ok()?;
    let response = String::from_utf8_lossy(&response.body);
    Some(response.to_string())
}

pub async fn get_latest_version() -> Option<GitHubApiResponse> {
    let request = create_api_check_endpoint();
    let response = fetch_response(request).await?;
    GitHubApiResponse::from_api_response(&response)
}
