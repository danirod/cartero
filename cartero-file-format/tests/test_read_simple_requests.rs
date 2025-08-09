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
use cartero_interop::FileLoadError;
use cartero_objects::RequestMethod;

#[test]
fn simple_request() {
    let content = include_str!("simple_requests/simple_request.cartero");
    let result = deserialize_request(content);
    let result = result.unwrap();
    assert!(result.warnings().is_empty());
    let object = result.object();
    assert_eq!(object.url(), "https://www.example.com/api/users");
    assert_eq!(object.method(), RequestMethod::Put);
}

#[test]
fn invalid_version() {
    let content = include_str!("simple_requests/invalid_version.cartero");
    let result = deserialize_request(content);
    match result {
        Err(FileLoadError::SchemaTooNew) => {}
        _ => panic!("expected version not to be valid"),
    }
}

#[test]
fn version_too_new() {
    let content = include_str!("simple_requests/version_too_new.cartero");
    let result = deserialize_request(content);
    match result {
        Err(FileLoadError::SchemaTooNew) => {}
        _ => panic!("expected version not to be valid"),
    }
}

#[test]
fn no_version() {
    let content = include_str!("simple_requests/no_version.cartero");
    let result = deserialize_request(content);
    match result {
        Err(FileLoadError::DeserializationError(e)) => {
            let msg = e.to_string();
            assert!(msg.contains("version"), "{}", msg);
        }
        _ => panic!("expected version not to be valid"),
    }
}

#[test]
fn no_url() {
    let content = include_str!("simple_requests/no_url.cartero");
    let result = deserialize_request(content);
    match result {
        Err(FileLoadError::DeserializationError(e)) => {
            let msg = e.to_string();
            assert!(msg.contains("url"), "{}", msg);
        }
        _ => panic!("expected version not to be valid"),
    }
}

#[test]
fn no_method() {
    let content = include_str!("simple_requests/no_method.cartero");
    let result = deserialize_request(content);
    match result {
        Err(FileLoadError::DeserializationError(e)) => {
            let msg = e.to_string();
            assert!(msg.contains("method"), "{}", msg);
        }
        _ => panic!("expected version not to be valid"),
    }
}

#[test]
fn fallbacks_to_get() {
    let content = include_str!("simple_requests/fallbacks_to_get.cartero");
    let result = deserialize_request(content);
    let result = result.unwrap();
    let warnings = result.warnings();
    assert_eq!(1, warnings.len());
    assert!(match &warnings[0] {
        cartero_interop::FileWarningTag::InvalidHttpVerb(verb) => verb == "HELLO",
    });
    let object = result.object();
    assert_eq!(object.url(), "https://www.example.com/api/users");
    assert_eq!(object.method(), RequestMethod::Get);
}
