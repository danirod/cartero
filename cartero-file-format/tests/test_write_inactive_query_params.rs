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
use cartero_objects::{Field, Request, RequestMethod};

#[test]
fn no_inactive_not_sync() {
    let req = Request::builder(
        "https://www.example.com/users?page=1&per_page=15&sort=-created&group=category",
        RequestMethod::Get,
    )
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("inactive_params/no_inactive.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn no_inactive() {
    let req = Request::builder(
        "https://www.example.com/users?page=1&per_page=15&sort=-created&group=category",
        RequestMethod::Get,
    )
    .param(&Field::builder().key("page").value("1").build())
    .param(&Field::builder().key("per_page").value("15").build())
    .param(&Field::builder().key("group").value("category").build())
    .param(&Field::builder().key("sort").value("-created").build())
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("inactive_params/no_inactive.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn inactive_not_sync() {
    let req = Request::builder(
        "https://www.example.com/users?page=1&per_page=15&group=category",
        RequestMethod::Get,
    )
    .param(
        &Field::builder()
            .key("sort")
            .value("-created")
            .active(false)
            .build(),
    )
    .param(
        &Field::builder()
            .key("utm_content")
            .value("test")
            .active(false)
            .build(),
    )
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("inactive_params/inactive.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn inactive() {
    let req = Request::builder(
        "https://www.example.com/users?page=1&per_page=15&group=category",
        RequestMethod::Get,
    )
    .param(
        &Field::builder()
            .key("sort")
            .value("-created")
            .active(false)
            .build(),
    )
    .param(&Field::builder().key("page").value("1").build())
    .param(&Field::builder().key("per_page").value("15").build())
    .param(
        &Field::builder()
            .key("utm_content")
            .value("test")
            .active(false)
            .build(),
    )
    .param(&Field::builder().key("group").value("category").build())
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("inactive_params/inactive.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn multiple_inactive_not_sync() {
    let req = Request::builder(
        "https://www.example.com/users?page=1&per_page=15&group=category",
        RequestMethod::Get,
    )
    .param(
        &Field::builder()
            .key("page")
            .value("2")
            .active(false)
            .masked(true)
            .build(),
    )
    .param(
        &Field::builder()
            .key("page")
            .value("3")
            .active(false)
            .build(),
    )
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("inactive_params/multiple_inactive.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn multiple_inactive() {
    let req = Request::builder(
        "https://www.example.com/users?page=1&per_page=15&group=category",
        RequestMethod::Get,
    )
    .param(&Field::builder().key("page").value("1").build())
    .param(&Field::builder().key("per_page").value("15").build())
    .param(
        &Field::builder()
            .key("page")
            .value("2")
            .active(false)
            .masked(true)
            .build(),
    )
    .param(&Field::builder().key("group").value("category").build())
    .param(
        &Field::builder()
            .key("page")
            .value("3")
            .active(false)
            .build(),
    )
    .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("inactive_params/multiple_inactive.cartero");
    assert_eq!(actual, encoded);
}
