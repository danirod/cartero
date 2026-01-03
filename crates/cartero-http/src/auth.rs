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

use cartero_objects::{Request, RequestAuthenticationDataExt};

use crate::RequestError;

pub(crate) struct BoundHeaders(Vec<(String, String)>);

impl BoundHeaders {
    pub fn headers(&self) -> Vec<(String, String)> {
        self.0.clone()
    }
}

impl TryFrom<&Request> for BoundHeaders {
    type Error = RequestError;

    fn try_from(value: &Request) -> Result<Self, Self::Error> {
        let value = value.resolve()?;
        let headers: Vec<(String, String)> = value
            .authentication()
            .auth_data()
            .map(|auth| auth.rendered_headers())
            .unwrap_or_default();
        Ok(BoundHeaders(headers))
    }
}

#[cfg(test)]
mod tests {
    use cartero_objects::{
        Request, RequestAuthentication, RequestAuthenticationBasic, RequestAuthenticationBearer,
        RequestMethod,
    };

    use crate::auth::BoundHeaders;

    #[test]
    fn test_authentication_none() {
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_auth(RequestAuthentication::builder().none().build())
            .build();

        let BoundHeaders(headers) = BoundHeaders::try_from(&req).unwrap();
        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn test_authentication_inherit() {
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_auth(RequestAuthentication::builder().inherit().build())
            .build();

        let BoundHeaders(headers) = BoundHeaders::try_from(&req).unwrap();
        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn test_authentication_token() {
        let bearer = RequestAuthenticationBearer::builder()
            .token("12341234")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_auth(
                RequestAuthentication::builder()
                    .bearer_token(&bearer)
                    .build(),
            )
            .build();

        let BoundHeaders(headers) = BoundHeaders::try_from(&req).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer 12341234");
    }

    #[test]
    fn test_authentication_basic() {
        let basic = RequestAuthenticationBasic::builder()
            .username("admin")
            .password("1234")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_auth(RequestAuthentication::builder().basic_auth(&basic).build())
            .build();

        let BoundHeaders(headers) = BoundHeaders::try_from(&req).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Basic YWRtaW46MTIzNA==");
    }
}
