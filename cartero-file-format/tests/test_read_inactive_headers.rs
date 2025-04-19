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

use cartero_file_format::deserialize_request;
use common::assert_field;
use gio::prelude::ListModelExt;

mod common;

#[test]
fn no_inactive() {
    let content = include_str!("inactive_params/no_inactive.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.params().n_items(), 4);
    let params = object.params().group_by_key();

    let page = params.get("page").unwrap();
    assert_eq!(page.len(), 1);
    assert_field(page[0].as_ref(), "page", "1", true, false);

    let per_page = params.get("per_page").unwrap();
    assert_eq!(per_page.len(), 1);
    assert_field(per_page[0].as_ref(), "per_page", "15", true, false);

    let sort: &Vec<cartero_objects::Field> = params.get("sort").unwrap();
    assert_eq!(sort.len(), 1);
    assert_field(sort[0].as_ref(), "sort", "-created", true, false);

    let group: &Vec<cartero_objects::Field> = params.get("group").unwrap();
    assert_eq!(group.len(), 1);
    assert_field(group[0].as_ref(), "group", "category", true, false);
}

#[test]
fn inactive() {
    let content = include_str!("inactive_params/inactive.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.params().n_items(), 5);
    let params = object.params().group_by_key();

    let page = params.get("page").unwrap();
    assert_eq!(page.len(), 1);
    assert_field(page[0].as_ref(), "page", "1", true, false);

    let per_page = params.get("per_page").unwrap();
    assert_eq!(per_page.len(), 1);
    assert_field(per_page[0].as_ref(), "per_page", "15", true, false);

    let sort: &Vec<cartero_objects::Field> = params.get("sort").unwrap();
    assert_eq!(sort.len(), 1);
    assert_field(sort[0].as_ref(), "sort", "-created", false, false);

    let group: &Vec<cartero_objects::Field> = params.get("group").unwrap();
    assert_eq!(group.len(), 1);
    assert_field(group[0].as_ref(), "group", "category", true, false);
}

#[test]
fn multiple_inactive() {
    let content = include_str!("inactive_params/multiple_inactive.cartero");
    let result = deserialize_request(content).unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();

    assert_eq!(object.params().n_items(), 5);
    let params = object.params().group_by_key();

    let page = params.get("page").unwrap();
    assert_eq!(page.len(), 3);
    assert_field(page[0].as_ref(), "page", "1", true, false);
    assert_field(page[1].as_ref(), "page", "2", false, true);
    assert_field(page[2].as_ref(), "page", "3", false, false);

    let group: &Vec<cartero_objects::Field> = params.get("group").unwrap();
    assert_eq!(group.len(), 1);
    assert_field(group[0].as_ref(), "group", "category", true, false);
}
