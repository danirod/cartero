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

use cartero_objects::{
    RequestAuthentication, RequestAuthenticationBasic, RequestAuthenticationBearer,
    RequestAuthenticationData, RequestAuthenticationType,
};
use glib::object::Cast;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum AuthorizationValue {
    #[serde(rename = "basic")]
    Basic { username: String, password: String },

    #[serde(rename = "bearer")]
    Bearer { token: String },
}

impl TryFrom<RequestAuthentication> for AuthorizationValue {
    type Error = ();

    fn try_from(value: RequestAuthentication) -> Result<Self, Self::Error> {
        match value.auth_type() {
            cartero_objects::RequestAuthenticationType::BasicAuth => {
                let auth = value.basic_auth().ok_or(())?;
                Ok(Self::Basic {
                    username: auth.username(),
                    password: auth.password(),
                })
            }
            cartero_objects::RequestAuthenticationType::BearerToken => {
                let token = value.bearer_token().ok_or(())?;
                Ok(Self::Bearer {
                    token: token.token(),
                })
            }
            _ => Err(()),
        }
    }
}

impl From<AuthorizationValue> for RequestAuthentication {
    fn from(value: AuthorizationValue) -> Self {
        let (auth_type, auth_data): (RequestAuthenticationType, RequestAuthenticationData) =
            match value {
                AuthorizationValue::Basic { username, password } => {
                    let basic = RequestAuthenticationBasic::new(username, password);
                    (RequestAuthenticationType::BasicAuth, basic.upcast())
                }
                AuthorizationValue::Bearer { token } => {
                    let bearer = RequestAuthenticationBearer::new(token);
                    (RequestAuthenticationType::BearerToken, bearer.upcast())
                }
            };
        RequestAuthentication::new(auth_type, Some(auth_data))
    }
}

#[cfg(test)]
mod tests {
    use cartero_objects::{
        RequestAuthentication, RequestAuthenticationBasic, RequestAuthenticationBearer,
        RequestAuthenticationType,
    };

    use super::AuthorizationValue;

    #[test]
    fn converts_from_basic_auth() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            Some(RequestAuthenticationBasic::new("user", "pass")),
        );

        let value = AuthorizationValue::try_from(auth).unwrap();
        let AuthorizationValue::Basic { username, password } = value else {
            panic!("Invalid type");
        };
        assert_eq!(username, "user");
        assert_eq!(password, "pass");
    }

    #[test]
    fn converts_from_bearer_token() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BearerToken,
            Some(RequestAuthenticationBearer::new("auth_token")),
        );

        let value = AuthorizationValue::try_from(auth).unwrap();
        let AuthorizationValue::Bearer { token } = value else {
            panic!("Invalid type");
        };
        assert_eq!(token, "auth_token");
    }

    #[test]
    #[should_panic]
    fn converts_from_unknown() {
        let auth = RequestAuthentication::default();
        AuthorizationValue::try_from(auth).unwrap();
    }

    #[test]
    fn convert_to_basic_auth() {
        let value = AuthorizationValue::Basic {
            username: "admin".into(),
            password: "1234".into(),
        };
        let object = RequestAuthentication::from(value);
        assert_eq!(object.auth_type(), RequestAuthenticationType::BasicAuth);
        assert_eq!(object.basic_auth().unwrap().username(), "admin");
        assert_eq!(object.basic_auth().unwrap().password(), "1234");
    }

    #[test]
    fn convert_to_bearer_token() {
        let value = AuthorizationValue::Bearer {
            token: "auth_token".into(),
        };
        let object = RequestAuthentication::from(value);
        assert_eq!(object.auth_type(), RequestAuthenticationType::BearerToken);
        assert_eq!(object.bearer_token().unwrap().token(), "auth_token");
    }
}
