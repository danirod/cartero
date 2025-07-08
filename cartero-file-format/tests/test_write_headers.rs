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
pub fn no_headers() {
    let request = Request::builder("https://www.example.com/api/users", RequestMethod::Put).build();

    let encoded = serialize_request(&request).unwrap();
    let file = include_str!("headers/no_headers.cartero");
    assert_eq!(file, encoded);
}

#[test]
pub fn headers() {
    let request = Request::builder("https://www.example.com/api/users", RequestMethod::Put)
        .header(
            &Field::builder()
                .key("Content-Type")
                .value("text/html")
                .active(false)
                .build(),
        )
        .header(
            &Field::builder()
                .key("Authorization")
                .value("Bearer token1234")
                .masked(true)
                .build(),
        )
        .header(
            &Field::builder()
                .key("Accept-Language")
                .value("en-US")
                .build(),
        )
        .header(
            &Field::builder()
                .key("X-Api-Key")
                .value("apikey-1234")
                .active(false)
                .masked(true)
                .build(),
        )
        .header(&Field::builder().key("Accept").value("text/html").build())
        .build();

    let encoded = serialize_request(&request).unwrap();
    let file = include_str!("headers/headers.cartero");
    assert_eq!(file, encoded);
}

#[test]
pub fn dupe_headers() {
    let request = Request::builder("https://www.example.com/api/users", RequestMethod::Put)
        .header(
            &Field::builder()
                .key("Accept-Language")
                .value("en-US")
                .build(),
        )
        .header(
            &Field::builder()
                .key("Accept-Language")
                .value("es-ES")
                .active(false)
                .build(),
        )
        .header(
            &Field::builder()
                .key("X-Forwarded-Host")
                .value("192.168.1.1")
                .build(),
        )
        .header(
            &Field::builder()
                .key("X-Forwarded-Host")
                .value("192.168.33.10")
                .active(false)
                .build(),
        )
        .header(
            &Field::builder()
                .key("X-Forwarded-Host")
                .value("10.0.2.15")
                .masked(true)
                .build(),
        )
        .header(
            &Field::builder()
                .key("X-Forwarded-Host")
                .value("172.16.99.15")
                .active(false)
                .masked(true)
                .build(),
        )
        .build();

    let encoded = serialize_request(&request).unwrap();
    let file = include_str!("headers/dupe_headers.cartero");
    assert_eq!(file, encoded);
}
