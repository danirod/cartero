// Copyright 2024-2026 the Cartero authors
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

use url::Url;

use crate::RequestError;

pub(crate) fn normalize_url(url: &str) -> Result<String, RequestError> {
    if !url.contains("://") {
        return Err(RequestError::MissingProtocol);
    }
    match Url::parse(url) {
        Ok(url) => {
            // Check for protocol as well.
            match url.scheme() {
                "http" | "https" => Ok(url.to_string()),
                other => Err(RequestError::UnsupportedProtocol(other.to_owned())),
            }
        }
        Err(url::ParseError::RelativeUrlWithoutBase) => Err(RequestError::MissingProtocol),
        Err(_) => Err(RequestError::UrlBadParse),
    }
}

#[cfg(test)]
mod tests {
    use crate::{RequestError, url::normalize_url};

    #[test]
    pub fn test_url_normalization() {
        let positive = vec![
            // These are straightforward.
            "https://example.com/api/users",
            "http://example.com/api/users",
            "http://example.com:8080/api/users",
            "https://example.com:8043/api/users",
            // IP address
            "https://192.168.1.4/api/users",
            "http://192.168.1.4/api/users",
            "http://192.168.1.4",
            "https://192.168.1.4",
            "http://192.168.1.4:4000",
            "https://192.168.1.4:4000/api/users",
            // These should work too.
            "https://localhost:3000/api/users",
            "http://localhost:3000/api/users",
            "http://localhost:3000",
            "http://localhost",
            "http://localhost/api/users",
            // Basic authentication in the URL
            "http://admin:admin@router/login",
            "http://admin:admin@router.home/login",
            "http://admin:admin@192.168.1.1/login",
        ];

        let wrong_protocol = vec!["gemini://geminiprotocol.net", "ws://localhost:8000/socket"];

        let missing_protocol = vec![
            // Typical use in development
            "example.com/api/users",
            "example.com",
            "example.com:8000",
            "192.168.1.4",
            "192.168.1.4/api/users",
            "192.168.1.4:8000",
            "localhost",
            "localhost/users",
            "localhost:3000/users",
            // This is open to discussion but for this use case, it's an HTTP request to example.com
            // using user=mailto and pass=foobar as a basic authentication, and not an email request.
            "mailto:foobar@example.com",
            "mailto:foobar@example.com/api/users",
        ];

        for url in positive {
            let result = normalize_url(url);
            assert!(
                result.is_ok(),
                "{} should have been accepted, was {:?}",
                url,
                result
            );
        }

        for url in wrong_protocol {
            let result = normalize_url(url);
            let Err(RequestError::UnsupportedProtocol(_)) = result else {
                panic!("{} should have been an unsupported protocol", url);
            };
        }

        for url in missing_protocol {
            let result = normalize_url(url);
            assert!(result.is_err());
        }
    }

    #[test]
    pub fn test_remove_leading_space() {
        let url = " https://example.com/api/v1/users";
        let actual = normalize_url(url).unwrap();
        assert_eq!(actual, "https://example.com/api/v1/users");
    }

    #[test]
    pub fn test_remove_trailing_space() {
        let url = "https://example.com/api/v1/users ";
        let actual = normalize_url(url).unwrap();
        assert_eq!(actual, "https://example.com/api/v1/users");
    }

    #[test]
    pub fn test_remove_leading_and_trailing_space() {
        let url = " https://example.com/api/v1/users ";
        let actual = normalize_url(url).unwrap();
        assert_eq!(actual, "https://example.com/api/v1/users");
    }
}
