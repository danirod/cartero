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

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum RequestAuthorization {
    #[default]
    None,
    Basic {
        username: String,
        password: String,
    },
    Bearer(String),
}

impl Into<cartero_objects::RequestAuthentication> for RequestAuthorization {
    fn into(self) -> cartero_objects::RequestAuthentication {
        match self {
            RequestAuthorization::None => cartero_objects::RequestAuthentication::builder()
                .none()
                .build(),
            RequestAuthorization::Basic { username, password } => {
                let basic = cartero_objects::RequestAuthenticationBasic::builder()
                    .username(username)
                    .password(password)
                    .build();
                cartero_objects::RequestAuthentication::builder()
                    .basic_auth(&basic)
                    .build()
            }
            RequestAuthorization::Bearer(token) => {
                let bearer = cartero_objects::RequestAuthenticationBearer::builder()
                    .token(token)
                    .build();
                cartero_objects::RequestAuthentication::builder()
                    .bearer_token(&bearer)
                    .build()
            }
        }
    }
}

impl From<cartero_objects::RequestAuthentication> for RequestAuthorization {
    fn from(value: cartero_objects::RequestAuthentication) -> Self {
        match value.auth_type() {
            cartero_objects::RequestAuthenticationType::None => Self::None,
            cartero_objects::RequestAuthenticationType::Inherit => Self::None,
            cartero_objects::RequestAuthenticationType::BasicAuth => {
                let basic_auth = value.basic_auth().unwrap();
                Self::Basic {
                    username: basic_auth.username(),
                    password: basic_auth.password(),
                }
            }
            cartero_objects::RequestAuthenticationType::BearerToken => {
                let bearer_token = value.bearer_token().unwrap();
                Self::Bearer(bearer_token.token())
            }
        }
    }
}
