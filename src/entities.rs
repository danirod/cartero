// Copyright 2024 the Cartero authors
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

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use srtemplate::SrTemplate;

use crate::objects::KeyValueItem;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyValue {
    pub name: String,
    pub value: String,
    pub active: bool,
    pub secret: bool,
}

impl PartialOrd for KeyValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for KeyValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl From<(String, String)> for KeyValue {
    fn from(value: (String, String)) -> Self {
        let (name, value) = value;
        KeyValue {
            name,
            value,
            active: true,
            secret: false,
        }
    }
}

impl From<(&str, &str)> for KeyValue {
    fn from(value: (&str, &str)) -> Self {
        let (k, v) = value;
        KeyValue {
            name: k.into(),
            value: v.into(),
            active: true,
            secret: false,
        }
    }
}

impl From<KeyValueItem> for KeyValue {
    fn from(value: KeyValueItem) -> Self {
        Self {
            name: value.header_name().clone(),
            value: value.header_value().clone(),
            active: value.active(),
            secret: value.secret(),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, glib::Boxed)]
#[boxed_type(name = "GKeyValueTable")]
pub struct KeyValueTable(Vec<KeyValue>);

impl KeyValueTable {
    pub fn new(entries: &[KeyValue]) -> Self {
        Self(entries.to_vec())
    }

    pub fn group_by(&self) -> HashMap<String, Vec<KeyValue>> {
        let mut hash_map: HashMap<String, Vec<KeyValue>> = HashMap::new();
        for row in &self.0 {
            hash_map
                .entry(row.name.clone())
                .or_default()
                .push(row.clone());
        }
        hash_map
    }

    pub fn header(&self, key: &str) -> Option<Vec<&str>> {
        let compare_key: String = key.to_lowercase();
        let mut headers: Vec<&str> = self
            .0
            .iter()
            .filter_map(|kv| {
                if kv.name.to_lowercase() == compare_key {
                    Some(kv.value.as_str())
                } else {
                    None
                }
            })
            .collect();
        headers.sort();
        if headers.is_empty() {
            None
        } else {
            Some(headers)
        }
    }
}

impl Deref for KeyValueTable {
    type Target = Vec<KeyValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyValueTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<A> FromIterator<A> for KeyValueTable
where
    Vec<KeyValue>: FromIterator<A>,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum RequestMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
    Trace,
}

impl TryFrom<&str> for RequestMethod {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "get" => Ok(RequestMethod::Get),
            "post" => Ok(RequestMethod::Post),
            "put" => Ok(RequestMethod::Put),
            "patch" => Ok(RequestMethod::Patch),
            "delete" => Ok(RequestMethod::Delete),
            "options" => Ok(RequestMethod::Options),
            "head" => Ok(RequestMethod::Head),
            "trace" => Ok(RequestMethod::Trace),
            _ => Err(()),
        }
    }
}

impl From<RequestMethod> for &str {
    fn from(val: RequestMethod) -> Self {
        match val {
            RequestMethod::Get => "GET",
            RequestMethod::Post => "POST",
            RequestMethod::Put => "PUT",
            RequestMethod::Patch => "PATCH",
            RequestMethod::Delete => "DELETE",
            RequestMethod::Head => "HEAD",
            RequestMethod::Options => "OPTIONS",
            RequestMethod::Trace => "TRACE",
        }
    }
}

impl From<RequestMethod> for String {
    fn from(value: RequestMethod) -> String {
        let string: &str = value.into();
        String::from(string)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum RawEncoding {
    Json,
    Xml,
    #[default]
    OctetStream,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum RequestPayload {
    #[default]
    None,
    Urlencoded(KeyValueTable),
    Multipart {
        params: KeyValueTable,
    },
    Raw {
        encoding: RawEncoding,
        content: Vec<u8>,
    },
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EndpointData {
    pub url: String,
    pub method: RequestMethod,
    pub headers: KeyValueTable,
    pub variables: KeyValueTable,
    pub body: RequestPayload,
}

impl EndpointData {
    pub fn template_processor(&self) -> SrTemplate {
        let context = SrTemplate::default();
        for item in self.variables.iter() {
            context.add_variable(item.name.clone(), &item.value);
        }
        context
    }

    pub fn process_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for item in self.headers.iter() {
            if item.active {
                headers.insert(item.name.clone(), item.value.clone());
            }
        }
        headers
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResponseData {
    pub status_code: u32,
    pub duration: u128,
    pub size: usize,
    pub headers: KeyValueTable,
    pub body: Vec<u8>,
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
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn seconds(&self) -> String {
        let seconds = (self.duration as f64) / 1000.0;
        format!("{seconds}")
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::{KeyValue, RequestMethod};

    use super::{KeyValueTable, ResponseData};

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
                headers: KeyValueTable(vec![header]),
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
                headers: KeyValueTable(vec![header]),
                body: Vec::new(),
            };
            assert_eq!(response.is_xml(), expected);
        }
    }

    #[test]
    pub fn test_convert_str_to_method() {
        assert!(RequestMethod::try_from("GET").is_ok_and(|x| x == RequestMethod::Get));
        assert!(RequestMethod::try_from("post").is_ok_and(|x| x == RequestMethod::Post));
        assert!(RequestMethod::try_from("Patch").is_ok_and(|x| x == RequestMethod::Patch));
        assert!(RequestMethod::try_from("Juan").is_err());
    }

    #[test]
    fn test_key_value_table_header() {
        let headers = vec![
            ("Content-Type", "application/json").into(),
            ("Set-Cookie", "cookie2=value2").into(),
            ("Set-Cookie", "cookie1=value1").into(),
        ];
        let table = KeyValueTable(headers);

        let ctype = table.header("content-type");
        assert_eq!(ctype, Some(vec!["application/json"]));

        let cookie = table.header("Set-cookie");
        assert_eq!(cookie, Some(vec!["cookie1=value1", "cookie2=value2"]));

        let empty = table.header("Accept");
        assert_eq!(empty, None);
    }

    #[test]
    fn test_group_by_table_header() {
        let headers = vec![
            ("Content-Type", "application/json").into(),
            ("Set-Cookie", "cookie2=value2").into(),
            ("Set-Cookie", "cookie1=value1").into(),
        ];
        let table = KeyValueTable(headers);

        let grouped = table.group_by();
        assert_eq!(grouped.len(), 2);

        let ctype = grouped.get("Content-Type").unwrap();
        assert_eq!(
            ctype,
            &vec![KeyValue {
                name: "Content-Type".into(),
                value: "application/json".into(),
                active: true,
                secret: false
            },]
        );

        let cookies = grouped.get("Set-Cookie").unwrap();
        assert_eq!(
            cookies,
            &vec![
                KeyValue {
                    name: "Set-Cookie".into(),
                    value: "cookie2=value2".into(),
                    active: true,
                    secret: false
                },
                KeyValue {
                    name: "Set-Cookie".into(),
                    value: "cookie1=value1".into(),
                    active: true,
                    secret: false
                },
            ]
        );
    }
}
