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
use cartero_objects::{RequestBodyRawType, RequestBodyType};
use common::assert_field;
use gio::prelude::ListModelExt;

#[test]
pub fn none() {
    let contents = include_str!("body/none.cartero");
    let result = deserialize_request(contents).unwrap();
    assert!(result.warnings().is_empty());
    let request = result.object();
    assert_eq!(request.body().body_type(), RequestBodyType::None);
}

#[test]
pub fn unknown() {
    let contents = include_str!("body/unknown.cartero");
    let result = deserialize_request(contents);
    assert!(result.is_err());
}

#[test]
pub fn legacy_format() {
    let contents = include_str!("body/legacy_format.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Raw);
    let raw = request.body().raw().unwrap();
    assert_eq!(raw.payload_type(), RequestBodyRawType::OctetStream);
    assert_eq!(raw.payload(), "request contents");
}

#[test]
pub fn url_encoded() {
    let contents = include_str!("body/url_encoded.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::UrlEncoded);
    let urlenc = request.body().urlencoded().unwrap();
    assert_eq!(urlenc.params().n_items(), 4);
    let vars = urlenc.params().group_by_key();

    let username = vars.get("username").unwrap();
    assert_eq!(username.len(), 1);
    assert_field(username[0].as_ref(), "username", "foo", true, false);

    let password = vars.get("password").unwrap();
    assert_eq!(password.len(), 1);
    assert_field(password[0].as_ref(), "password", "bar", true, true);

    let remember_me = vars.get("remember_me").unwrap();
    assert_eq!(remember_me.len(), 1);
    assert_field(remember_me[0].as_ref(), "remember_me", "1", false, false);

    let csrf_token = vars.get("csrf_token").unwrap();
    assert_eq!(csrf_token.len(), 1);
    assert_field(csrf_token[0].as_ref(), "csrf_token", "token", false, true);
}

#[test]
pub fn url_encoded_duplicate() {
    let contents = include_str!("body/url_encoded_duplicate.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::UrlEncoded);
    let urlenc = request.body().urlencoded().unwrap();
    assert_eq!(urlenc.params().n_items(), 4);
    let vars = urlenc.params().group_by_key();

    let category = vars.get("category").unwrap();
    assert_eq!(category.len(), 4);
    assert_field(category[0].as_ref(), "category", "10", true, false);
    assert_field(category[1].as_ref(), "category", "20", false, false);
    assert_field(category[2].as_ref(), "category", "30", false, true);
    assert_field(category[3].as_ref(), "category", "40", true, true);
}

#[test]
pub fn url_encoded_combined() {
    let contents = include_str!("body/url_encoded_combined.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::UrlEncoded);
    let urlenc = request.body().urlencoded().unwrap();
    assert_eq!(urlenc.params().n_items(), 5);
    let vars = urlenc.params().group_by_key();

    let category = vars.get("category").unwrap();
    assert_eq!(category.len(), 3);
    assert_field(category[0].as_ref(), "category", "kitchenware", true, false);
    assert_field(category[1].as_ref(), "category", "decoration", true, false);
    assert_field(category[2].as_ref(), "category", "garden", true, false);

    let max_price = vars.get("max_price").unwrap();
    assert_eq!(max_price.len(), 1);
    assert_field(max_price[0].as_ref(), "max_price", "25", true, false);

    let min_price = vars.get("min_price").unwrap();
    assert_eq!(min_price.len(), 1);
    assert_field(min_price[0].as_ref(), "min_price", "10", true, false);
}

#[test]
pub fn multipart() {
    let contents = include_str!("body/multipart.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Multipart);
    let multipart = request.body().multipart().unwrap();
    assert_eq!(multipart.params().n_items(), 4);
    let vars = multipart.params().group_by_key();

    let username = vars.get("username").unwrap();
    assert_eq!(username.len(), 1);
    assert_field(username[0].as_ref(), "username", "foo", true, false);

    let password = vars.get("password").unwrap();
    assert_eq!(password.len(), 1);
    assert_field(password[0].as_ref(), "password", "bar", true, true);

    let remember_me = vars.get("remember_me").unwrap();
    assert_eq!(remember_me.len(), 1);
    assert_field(remember_me[0].as_ref(), "remember_me", "1", false, false);

    let csrf_token = vars.get("csrf_token").unwrap();
    assert_eq!(csrf_token.len(), 1);
    assert_field(csrf_token[0].as_ref(), "csrf_token", "token", false, true);
}

#[test]
pub fn multipart_duplicate() {
    let contents = include_str!("body/multipart_duplicate.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Multipart);
    let multipart = request.body().multipart().unwrap();
    assert_eq!(multipart.params().n_items(), 4);
    let vars = multipart.params().group_by_key();

    let category = vars.get("category").unwrap();
    assert_eq!(category.len(), 4);
    assert_field(category[0].as_ref(), "category", "10", true, false);
    assert_field(category[1].as_ref(), "category", "20", false, false);
    assert_field(category[2].as_ref(), "category", "30", false, true);
    assert_field(category[3].as_ref(), "category", "40", true, true);
}

#[test]
pub fn multipart_combined() {
    let contents = include_str!("body/multipart_combined.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Multipart);
    let urlenc = request.body().multipart().unwrap();
    assert_eq!(urlenc.params().n_items(), 5);
    let vars = urlenc.params().group_by_key();

    let category = vars.get("category").unwrap();
    assert_eq!(category.len(), 3);
    assert_field(category[0].as_ref(), "category", "kitchenware", true, false);
    assert_field(category[1].as_ref(), "category", "decoration", true, false);
    assert_field(category[2].as_ref(), "category", "garden", true, false);

    let max_price = vars.get("max_price").unwrap();
    assert_eq!(max_price.len(), 1);
    assert_field(max_price[0].as_ref(), "max_price", "25", true, false);

    let min_price = vars.get("min_price").unwrap();
    assert_eq!(min_price.len(), 1);
    assert_field(min_price[0].as_ref(), "min_price", "10", true, false);
}

#[test]
pub fn raw() {
    let contents = include_str!("body/raw.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Raw);
    let raw = request.body().raw().unwrap();
    assert_eq!(raw.payload_type(), RequestBodyRawType::OctetStream);
    assert_eq!(raw.payload(), "this is raw content");
}

#[test]
pub fn octet_stream() {
    let contents = include_str!("body/octet_stream.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Raw);
    let raw = request.body().raw().unwrap();
    assert_eq!(raw.payload_type(), RequestBodyRawType::OctetStream);
    assert_eq!(raw.payload(), "this is raw content");
}

#[test]
pub fn xml() {
    let contents = include_str!("body/xml.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Raw);
    let raw = request.body().raw().unwrap();
    assert_eq!(raw.payload_type(), RequestBodyRawType::Xml);
    assert_eq!(
        raw.payload(),
        "<?xml version=\"1.0\" ?><payload value=\"Hello world\" />"
    );
}

#[test]
pub fn json() {
    let contents = include_str!("body/json.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Raw);
    let raw = request.body().raw().unwrap();
    assert_eq!(raw.payload_type(), RequestBodyRawType::Json);
    assert_eq!(raw.payload(), "{\"message\": \"hello world\"}");
}

#[test]
pub fn multiline_json() {
    let contents = include_str!("body/multiline_json.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::Raw);
    let raw = request.body().raw().unwrap();
    assert_eq!(raw.payload_type(), RequestBodyRawType::Json);
    assert_eq!(
        raw.payload(),
        r#"{
  "message": {
    "code": "HELLO_WORLD",
    "text": "Hello world!"
  }
}"#
    );
}

#[test]
pub fn file() {
    let contents = include_str!("body/file.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::File);
    let file = request.body().file().unwrap();
    assert_eq!(file.path(), "payload.xml");
    assert!(file.content_type().is_none());
}

#[test]
pub fn file_with_content_type() {
    let contents = include_str!("body/file_with_content_type.cartero");
    let result = deserialize_request(contents).unwrap();

    assert!(result.warnings().is_empty());
    let request = result.object();

    assert_eq!(request.body().body_type(), RequestBodyType::File);
    let file = request.body().file().unwrap();
    assert_eq!(file.path(), "payload.xml");
    assert!(file.content_type().is_some_and(|f| f == "application/xml"));
}
