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

use cartero_file_format::deserialize_request;
use cartero_objects::RequestAuthenticationType;

#[test]
fn none() {
    let contents = include_str!("authentication/none.cartero");
    let result = deserialize_request(contents).unwrap();
    let authentication = result.object().authentication();
    assert_eq!(authentication.auth_type(), RequestAuthenticationType::None);
}

#[test]
fn unknown() {
    let contents = include_str!("authentication/unknown.cartero");
    let result = deserialize_request(contents);
    assert!(result.is_err());
}

#[test]
fn basic_auth() {
    let contents = include_str!("authentication/basic_auth.cartero");
    let result = deserialize_request(contents).unwrap();
    let authentication = result.object().authentication();
    assert_eq!(
        authentication.auth_type(),
        RequestAuthenticationType::BasicAuth
    );
    let basic = authentication.basic_auth().unwrap();
    assert_eq!(basic.username(), "operator");
    assert_eq!(basic.password(), "password");
}

#[test]
fn bearer_token() {
    let contents = include_str!("authentication/bearer.cartero");
    let result = deserialize_request(contents).unwrap();
    let authentication = result.object().authentication();
    assert_eq!(
        authentication.auth_type(),
        RequestAuthenticationType::BearerToken
    );
    let basic = authentication.bearer_token().unwrap();
    assert_eq!(basic.token(), "auth1234");
}
