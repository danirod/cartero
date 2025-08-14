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
    Field, FieldTable, Request, RequestBody, RequestBodyFile, RequestBodyMultipart, RequestBodyRaw,
    RequestBodyRawType, RequestBodyType, RequestBodyUrlencoded, RequestMethod,
};

#[test]
pub fn json() {
    let req: Request = glib::Object::builder()
        .property("url", "https://www.example.com/login")
        .property("method", RequestMethod::Post)
        .build();

    let text = "{\"message\": \"hello world\"}";
    let json_body = RequestBodyRaw::new(cartero_objects::RequestBodyRawType::Json, text);
    req.body().set_body_type(RequestBodyType::Raw);
    req.body().set_body_data(Some(json_body.as_ref()));

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/json.cartero");
    assert_eq!(actual, encoded);
}

#[test]
pub fn multiline_json() {
    let req: Request = glib::Object::builder()
        .property("url", "https://www.example.com/login")
        .property("method", RequestMethod::Post)
        .build();

    let text = r#"{
  "message": {
    "code": "HELLO_WORLD",
    "text": "Hello world!"
  }
}"#;
    let json_body = RequestBodyRaw::new(cartero_objects::RequestBodyRawType::Json, text);
    req.body().set_body_type(RequestBodyType::Raw);
    req.body().set_body_data(Some(json_body.as_ref()));

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/multiline_json.cartero");
    assert_eq!(actual, encoded);
}

#[test]
pub fn multipart() {
    let fields = vec![
        Field::builder().key("username").value("foo").build(),
        Field::builder()
            .key("password")
            .value("bar")
            .masked(true)
            .build(),
        Field::builder()
            .key("remember_me")
            .value("1")
            .active(false)
            .build(),
        Field::builder()
            .key("csrf_token")
            .value("token")
            .active(false)
            .masked(true)
            .build(),
    ];
    let fields_table = FieldTable::from_iter(fields);
    let multipart = RequestBodyMultipart::builder()
        .params(&fields_table)
        .build();
    let req: Request = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(RequestBody::builder().multipart(&multipart).build())
        .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/multipart.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn multipart_duplicate() {
    let fields = vec![
        Field::builder().key("category").value("10").build(),
        Field::builder()
            .key("category")
            .value("20")
            .active(false)
            .build(),
        Field::builder()
            .key("category")
            .value("30")
            .active(false)
            .masked(true)
            .build(),
        Field::builder()
            .key("category")
            .value("40")
            .masked(true)
            .build(),
    ];
    let fields_table = FieldTable::from_iter(fields);
    let multipart = RequestBodyMultipart::builder()
        .params(&fields_table)
        .build();
    let req: Request = Request::builder("https://www.example.com/search", RequestMethod::Post)
        .with_body(RequestBody::builder().multipart(&multipart).build())
        .build();

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/multipart_duplicate.cartero");
    assert_eq!(actual, encoded);
}

#[test]
fn multipart_combined() {
    let fields = vec![
        Field::builder()
            .key("category")
            .value("kitchenware")
            .build(),
        Field::builder().key("category").value("decoration").build(),
        Field::builder().key("category").value("garden").build(),
        Field::builder().key("min_price").value("10").build(),
        Field::builder().key("max_price").value("25").build(),
    ];
    let fields_table = FieldTable::from_iter(fields);
    let req = Request::builder("https://www.example.com/search", RequestMethod::Put)
        .with_body(
            RequestBody::builder()
                .multipart(
                    &RequestBodyMultipart::builder()
                        .params(&fields_table)
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/multipart_combined.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn none() {
    let req = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(RequestBody::builder().none().build())
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/none.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn octet_stream() {
    let req = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(
            RequestBody::builder()
                .raw(
                    &RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
                        .payload("this is raw content")
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/octet_stream.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn urlencoded() {
    let req = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(
            RequestBody::builder()
                .urlencoded(
                    &RequestBodyUrlencoded::builder()
                        .field(&Field::builder().key("username").value("foo").build())
                        .field(
                            &Field::builder()
                                .key("password")
                                .value("bar")
                                .masked(true)
                                .build(),
                        )
                        .field(
                            &Field::builder()
                                .key("remember_me")
                                .value("1")
                                .active(false)
                                .build(),
                        )
                        .field(
                            &Field::builder()
                                .key("csrf_token")
                                .value("token")
                                .active(false)
                                .masked(true)
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/url_encoded.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn urlencoded_duplicate() {
    let fields = vec![
        Field::builder().key("category").value("10").build(),
        Field::builder()
            .key("category")
            .value("20")
            .active(false)
            .build(),
        Field::builder()
            .key("category")
            .value("30")
            .active(false)
            .masked(true)
            .build(),
        Field::builder()
            .key("category")
            .value("40")
            .masked(true)
            .build(),
    ];
    let fields_table = FieldTable::from_iter(fields);
    let req = Request::builder("https://www.example.com/search", RequestMethod::Post)
        .with_body(
            RequestBody::builder()
                .urlencoded(
                    &RequestBodyUrlencoded::builder()
                        .params(&fields_table)
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/url_encoded_duplicate.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn urlencoded_combined() {
    let fields = vec![
        Field::builder()
            .key("category")
            .value("kitchenware")
            .build(),
        Field::builder().key("category").value("decoration").build(),
        Field::builder().key("category").value("garden").build(),
        Field::builder().key("min_price").value("10").build(),
        Field::builder().key("max_price").value("25").build(),
    ];
    let fields_table = FieldTable::from_iter(fields);
    let req = Request::builder("https://www.example.com/search", RequestMethod::Put)
        .with_body(
            RequestBody::builder()
                .urlencoded(
                    &RequestBodyUrlencoded::builder()
                        .params(&fields_table)
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/url_encoded_combined.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn xml() {
    let req = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(
            RequestBody::builder()
                .raw(
                    &RequestBodyRaw::builder(RequestBodyRawType::Xml)
                        .payload("<?xml version=\"1.0\" ?><payload value=\"Hello world\" />")
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/xml.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn file() {
    let req = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(
            RequestBody::builder()
                .file(&RequestBodyFile::builder().path("payload.xml").build())
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/file.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn file_with_content_type() {
    let req = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(
            RequestBody::builder()
                .file(
                    &RequestBodyFile::builder()
                        .path("payload.xml")
                        .content_type(Some("application/xml"))
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/file_with_content_type.cartero");
    assert_eq!(encoded, actual);
}

#[test]
fn file_with_empty_content_type() {
    let req = Request::builder("https://www.example.com/login", RequestMethod::Post)
        .with_body(
            RequestBody::builder()
                .file(
                    &RequestBodyFile::builder()
                        .path("payload.xml")
                        .content_type(Some(""))
                        .build(),
                )
                .build(),
        )
        .build();
    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/file.cartero");
    assert_eq!(encoded, actual);
}
