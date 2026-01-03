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

mod common;

use cartero_file_format::deserialize_request;
use common::assert_field;
use gio::prelude::ListModelExt;

#[test]
fn no_headers() {
    let content = include_str!("headers/no_headers.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.headers().n_items(), 0);
}

#[test]
fn headers() {
    let content = include_str!("headers/headers.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.headers().n_items(), 5);
    let headers = object.headers().group_by_key();

    let language = headers.get("Accept-Language").unwrap();
    assert_eq!(language.len(), 1);
    assert_field(
        language[0].as_ref(),
        "Accept-Language",
        "en-US",
        true,
        false,
    );

    let accept = headers.get("Accept").unwrap();
    assert_field(accept[0].as_ref(), "Accept", "text/html", true, false);

    let authorization = headers.get("Authorization").unwrap();
    assert_field(
        authorization[0].as_ref(),
        "Authorization",
        "Bearer token1234",
        true,
        true,
    );

    let content_type = headers.get("Content-Type").unwrap();
    assert_field(
        content_type[0].as_ref(),
        "Content-Type",
        "text/html",
        false,
        false,
    );

    let api_key = headers.get("X-Api-Key").unwrap();
    assert_field(api_key[0].as_ref(), "X-Api-Key", "apikey-1234", false, true);
}

#[test]
fn dupe_headers() {
    let content = include_str!("headers/dupe_headers.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.headers().n_items(), 6);
    let headers = object.headers().group_by_key();

    let language = headers.get("Accept-Language").unwrap();
    assert_eq!(language.len(), 2);
    assert_field(
        language[0].as_ref(),
        "Accept-Language",
        "en-US",
        true,
        false,
    );
    assert_field(
        language[1].as_ref(),
        "Accept-Language",
        "es-ES",
        false,
        false,
    );

    let host = headers.get("X-Forwarded-Host").unwrap();
    assert_eq!(host.len(), 4);
    assert_field(
        host[0].as_ref(),
        "X-Forwarded-Host",
        "192.168.1.1",
        true,
        false,
    );
    assert_field(
        host[1].as_ref(),
        "X-Forwarded-Host",
        "192.168.33.10",
        false,
        false,
    );
    assert_field(
        host[2].as_ref(),
        "X-Forwarded-Host",
        "10.0.2.15",
        true,
        true,
    );
    assert_field(
        host[3].as_ref(),
        "X-Forwarded-Host",
        "172.16.99.15",
        false,
        true,
    );
}
