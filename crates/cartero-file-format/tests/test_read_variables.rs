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
fn no_variables() {
    let content = include_str!("variables/no_variables.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.variables().n_items(), 0);
}

#[test]
fn variables() {
    let content = include_str!("variables/variables.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.variables().n_items(), 2);
    let variables = object.variables().group_by_key();

    let root = variables.get("API_ROOT").unwrap();
    assert_eq!(root.len(), 1);
    assert_field(
        root[0].as_ref(),
        "API_ROOT",
        "https://www.example.com",
        true,
        false,
    );

    let page = variables.get("PAGE").unwrap();
    assert_eq!(page.len(), 1);
    assert_field(page[0].as_ref(), "PAGE", "2", true, false);
}

#[test]
fn dupe_variables() {
    let content = include_str!("variables/dupe_variables.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.variables().n_items(), 4);
    let variables = object.variables().group_by_key();

    let root = variables.get("API_ROOT").unwrap();
    assert_eq!(root.len(), 2);
    assert_field(
        root[0].as_ref(),
        "API_ROOT",
        "https://www.example.com",
        false,
        true,
    );
    assert_field(
        root[1].as_ref(),
        "API_ROOT",
        "http://localhost:3000",
        true,
        true,
    );

    let page = variables.get("PAGE").unwrap();
    assert_eq!(page.len(), 2);
    assert_field(page[0].as_ref(), "PAGE", "1", false, false);
    assert_field(page[1].as_ref(), "PAGE", "2", true, false);
}
