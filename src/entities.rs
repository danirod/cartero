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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyValue(pub (String, String));

impl PartialOrd for KeyValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for KeyValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let (this_h, this_v) = &self.0;
        let (other_h, other_v) = &other.0;
        let value = this_h.cmp(other_h);
        if value == std::cmp::Ordering::Equal {
            this_v.cmp(other_v)
        } else {
            value
        }
    }
}

impl From<(String, String)> for KeyValue {
    fn from(value: (String, String)) -> Self {
        KeyValue(value)
    }
}

impl From<(&str, &str)> for KeyValue {
    fn from(value: (&str, &str)) -> Self {
        let (k, v) = value;
        KeyValue((k.into(), v.into()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResponseData {
    pub status_code: u32,
    pub duration: u64,
    pub size: u64,
    pub headers: Vec<KeyValue>,
    pub body: Vec<u8>,
}

impl ResponseData {
    pub fn body_str(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn header(&self, key: &str) -> Option<Vec<&str>> {
        let compare_key = key.to_lowercase();
        let mut headers: Vec<&str> = self
            .headers
            .iter()
            .filter_map(|KeyValue((k, v))| {
                if k.to_lowercase() == compare_key {
                    Some(v.as_str())
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

#[cfg(test)]
mod tests {
    use super::ResponseData;

    #[test]
    fn test_header() {
        let response = ResponseData {
            status_code: 200,
            duration: 2400,
            size: 150,
            headers: vec![
                ("Content-Type", "application/json").into(),
                ("Set-Cookie", "cookie2=value2").into(),
                ("Set-Cookie", "cookie1=value1").into(),
            ],
            body: Vec::from(b"{}"),
        };

        let ctype = response.header("content-type");
        assert_eq!(ctype, Some(vec!["application/json"]));

        let cookie = response.header("Set-cookie");
        assert_eq!(cookie, Some(vec!["cookie1=value1", "cookie2=value2"]));

        let empty = response.header("Accept");
        assert_eq!(empty, None);
    }
}
