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
use cartero_objects::{Request, RequestBodyRaw, RequestBodyType, RequestMethod};

#[test]
pub fn json() {
    let req: Request = glib::Object::builder()
        .property("url", "https://www.example.com/login")
        .property("method", RequestMethod::Post)
        .build();

    let text = "{\"message\": \"hello world\"}";
    let payload = text.as_bytes();
    let json_body = RequestBodyRaw::new(cartero_objects::RequestBodyRawType::Json, payload);
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
    let payload = text.as_bytes();
    let json_body = RequestBodyRaw::new(cartero_objects::RequestBodyRawType::Json, payload);
    req.body().set_body_type(RequestBodyType::Raw);
    req.body().set_body_data(Some(json_body.as_ref()));

    let encoded = serialize_request(&req).unwrap();
    let actual = include_str!("body/multiline_json.cartero");
    assert_eq!(actual, encoded);
}
