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

use cartero_http::RequestEnvironment;
use cartero_objects::{Field, Request};
use serde::Deserialize;

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

const API_LATEST_URL: &'static str = "https://api.github.com/repos/danirod/cartero/releases/latest";

fn create_api_check_endpoint() -> Request {
    Request::builder(API_LATEST_URL, cartero_objects::RequestMethod::Get)
        .header(
            &Field::builder()
                .key("Accept")
                .value("application/json")
                .build(),
        )
        .build()
}

// Silently discards any error.
// TODO: at least could log it...
async fn fetch_response(payload: &Request) -> Option<String> {
    let environment = RequestEnvironment {
        config: cartero_http::ClientConfig {
            validate_tls: true,
            redirects: 0,
            timeout: 10.0,
        },
    };
    let response = cartero_isahc_client::request(&payload, &environment)
        .await
        .map_err(|e| {
            glib::g_debug!("Cartero", "fetch_response failed with error: {e:?}");
            e
        })
        .ok()?;
    response
        .body()
        .map(|body| String::from_utf8_lossy(&body).to_string())
}

pub async fn get_latest_version() -> Option<GitHubApiResponse> {
    let request = create_api_check_endpoint();
    glib::g_debug!(
        "Cartero",
        "Checking updates by poking the endpoint {}",
        &request.url(),
    );
    let response = fetch_response(&request).await?;
    let api_response = GitHubApiResponse::from_api_response(&response);
    glib::g_debug!(
        "Cartero",
        "Update query was resolved: {}",
        match &api_response {
            None => "Couldn't fetch response".to_string(),
            Some(resp) => format!("Latest version is {}", resp.get_latest_version()),
        }
    );
    api_response
}
