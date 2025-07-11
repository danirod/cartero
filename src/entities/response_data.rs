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

use formatx::formatx;
use gettextrs::gettext;

use super::KeyValueTable;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResponseData {
    pub status_code: u32,
    pub duration: u128,
    pub size: usize,
    pub headers: KeyValueTable,
    pub body: Vec<u8>,
}

impl From<cartero_objects::Response> for ResponseData {
    fn from(value: cartero_objects::Response) -> Self {
        Self {
            status_code: value.status_code(),
            duration: value.duration() as u128,
            size: value.size() as usize,
            headers: value.headers().into(),
            body: value.body().map(|bytes| bytes.to_vec()).unwrap_or_default(),
        }
    }
}

impl ResponseData {
    pub fn is_json(&self) -> bool {
        match self.headers.header("content-type") {
            Some(header) => match header[..] {
                [value] => value.contains("/json") || value.contains("+json"),
                _ => false,
            },
            None => false,
        }
    }

    pub fn is_xml(&self) -> bool {
        match self.headers.header("content-type") {
            Some(header) => match header[..] {
                [value] => value.contains("/xml") || value.contains("+xml"),
                _ => false,
            },
            None => false,
        }
    }

    pub fn body_str(&self) -> String {
        // gtksourceview breaks if the string is binary and there is a 00.
        // any other combination works fine and gets converted into a <?>,
        // but for x00 you get a GStrInteriorNulError instead
        String::from_utf8_lossy(&self.body)
            .into_owned()
            .replace("\x00", "�")
    }

    pub fn format_duration(&self) -> String {
        if self.duration >= 1000 {
            // Format as seconds
            let seconds = (self.duration as f64) / 1000.0;
            let duration = format!("{:.2}", seconds);
            // TRANSLATORS: duration measured in seconds, units in symbol, as in "1.23 s"
            formatx!(gettext("{} s"), duration).unwrap()
        } else {
            // Format as milliseconds.
            // TRANSLATORS: duration measured in milliseconds, as in "234 ms"
            formatx!(gettext("{} ms"), self.duration).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::{KeyValue, KeyValueTable, ResponseData};

    #[test]
    fn test_response_is_json() {
        let json_item = KeyValue::from(("Content-Type", "application/json"));
        let jsonld_item = KeyValue::from(("Content-Type", "application/ld+json; charset=utf8"));
        let textjson_item = KeyValue::from(("Content-Type", "text/json"));
        let vendor_item = KeyValue::from(("Content-Type", "application/vnd.github.raw+json"));
        let xml_item = KeyValue::from(("Content-Type", "application/xml"));
        let atom_item = KeyValue::from(("Content-Type", "application/atom+xml"));
        let jpeg_item = KeyValue::from(("Content-Type", "image/jpeg"));

        let cases = vec![
            (json_item, true),
            (jsonld_item, true),
            (textjson_item, true),
            (vendor_item, true),
            (xml_item, false),
            (atom_item, false),
            (jpeg_item, false),
        ];

        for (header, expected) in cases {
            let response = ResponseData {
                status_code: 200,
                duration: 0,
                size: 0,
                headers: KeyValueTable::new(&[header]),
                body: Vec::new(),
            };
            assert_eq!(response.is_json(), expected);
        }
    }

    #[test]
    fn test_response_is_xml() {
        let json_item = KeyValue::from(("Content-Type", "application/json"));
        let jsonld_item = KeyValue::from(("Content-Type", "application/ld+json; charset=utf8"));
        let textjson_item = KeyValue::from(("Content-Type", "text/json"));
        let vendor_item = KeyValue::from(("Content-Type", "application/vnd.github.raw+json"));
        let xml_item = KeyValue::from(("Content-Type", "application/xml"));
        let atom_item = KeyValue::from(("Content-Type", "application/atom+xml"));
        let jpeg_item = KeyValue::from(("Content-Type", "image/jpeg"));

        let cases = vec![
            (json_item, false),
            (jsonld_item, false),
            (textjson_item, false),
            (vendor_item, false),
            (xml_item, true),
            (atom_item, true),
            (jpeg_item, false),
        ];

        for (header, expected) in cases {
            let response = ResponseData {
                status_code: 200,
                duration: 0,
                size: 0,
                headers: KeyValueTable::new(&[header]),
                body: Vec::new(),
            };
            assert_eq!(response.is_xml(), expected);
        }
    }

    #[test]
    fn test_duration() {
        let cases = [
            (500u128, "500 ms"),
            (999u128, "999 ms"),
            (1000u128, "1.00 s"),
            (1234u128, "1.23 s"),
            (16774u128, "16.77 s"),
            (16775u128, "16.77 s"),
            (16776u128, "16.78 s"),
            (16777u128, "16.78 s"),
        ];
        for (input, expected) in cases {
            let response = ResponseData {
                status_code: 200,
                duration: input,
                size: 300,
                headers: KeyValueTable::default(),
                body: vec![],
            };
            let output = response.format_duration();
            assert_eq!(
                output, expected,
                "Expected {} to be formatted as '{}' (was '{}')",
                input, expected, output
            );
        }
    }
}
