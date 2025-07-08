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

use cartero_file_format::serialize_request;
use cartero_objects::{
    Request, RequestAuthenticationBasic, RequestAuthenticationBearer, RequestAuthenticationType,
    RequestMethod,
};

#[test]
pub fn none() {
    let req: Request = glib::Object::builder()
        .property("url", "https://www.example.com/login")
        .property("method", RequestMethod::Post)
        .build();
    let result = serialize_request(&req).unwrap();
    let expected = include_str!("authentication/none.cartero");
    assert_eq!(result, expected);
}

#[test]
pub fn basic_auth() {
    let req: Request = glib::Object::builder()
        .property("url", "https://www.example.com/dashboard")
        .property("method", RequestMethod::Get)
        .build();
    let auth_data = RequestAuthenticationBasic::new("operator", "password");
    req.authentication()
        .set_auth_type(RequestAuthenticationType::BasicAuth);
    req.authentication().set_auth_data(Some(auth_data.as_ref()));

    let result = serialize_request(&req).unwrap();
    let expected = include_str!("authentication/basic_auth.cartero");
    assert_eq!(result, expected);
}

#[test]
pub fn bearer() {
    let req: Request = glib::Object::builder()
        .property("url", "https://www.example.com/dashboard")
        .property("method", RequestMethod::Get)
        .build();
    let auth_data = RequestAuthenticationBearer::new("auth1234");
    req.authentication()
        .set_auth_type(RequestAuthenticationType::BearerToken);
    req.authentication().set_auth_data(Some(auth_data.as_ref()));

    let result = serialize_request(&req).unwrap();
    let expected = include_str!("authentication/bearer.cartero");
    assert_eq!(result, expected);
}
