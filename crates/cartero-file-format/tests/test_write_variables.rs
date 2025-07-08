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
use cartero_objects::{Field, Request};

#[test]
fn no_variables() {
    let req = Request::builder(
        "https://www.example.com/api/users",
        cartero_objects::RequestMethod::Put,
    )
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("variables/no_variables.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn variables() {
    let req = Request::builder(
        "{{API_ROOT}}/api/users?page={{PAGE}}",
        cartero_objects::RequestMethod::Put,
    )
    .variable(
        &Field::builder()
            .key("API_ROOT")
            .value("https://www.example.com")
            .build(),
    )
    .variable(&Field::builder().key("PAGE").value("2").build())
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("variables/variables.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn dupe_variables() {
    let req = Request::builder(
        "{{API_ROOT}}/api/users?page={{PAGE}}",
        cartero_objects::RequestMethod::Put,
    )
    .variable(
        &Field::builder()
            .key("API_ROOT")
            .value("https://www.example.com")
            .active(false)
            .masked(true)
            .build(),
    )
    .variable(
        &Field::builder()
            .key("API_ROOT")
            .value("http://localhost:3000")
            .masked(true)
            .build(),
    )
    .variable(
        &Field::builder()
            .key("PAGE")
            .value("1")
            .active(false)
            .build(),
    )
    .variable(&Field::builder().key("PAGE").value("2").build())
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("variables/dupe_variables.cartero");
    assert_eq!(actual, encoded);
}
