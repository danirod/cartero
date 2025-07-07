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
use cartero_objects::{Request, RequestMethod};

#[test]
fn simple_request() {
    let request: Request = glib::Object::builder()
        .property("url", "https://www.example.com/api/users")
        .property("method", RequestMethod::Put)
        .build();
    let result = serialize_request(&request).unwrap();
    let expected = include_str!("simple_requests/simple_request.cartero");
    assert_eq!(result, expected);
}
