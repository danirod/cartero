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

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use srtemplate::SrTemplate;

use super::KeyValue;

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

    /// Yields a new KeyValueTable where each value is interpolated according to the rules
    /// of the given renderer. If any value in the current KeyValueTable uses a variable,
    /// it will be interpolated.
    pub fn render(&self, renderer: &SrTemplate) -> Result<KeyValueTable, srtemplate::Error> {
        let entries = self
            .iter()
            .map(|var| {
                let name = renderer.render(var.name.clone())?;
                let value = renderer.render(var.value.clone())?;
                Ok(KeyValue {
                    name,
                    value,
                    active: var.active,
                    secret: var.secret,
                })
            })
            .collect::<Result<Vec<KeyValue>, srtemplate::Error>>()?;
        Ok(KeyValueTable::new(&entries))
    }

    /// Collects the active pairs of this KeyValueTable, excludes non-actives.
    pub fn to_active_pairs(&self) -> Vec<(String, String)> {
        self.iter()
            .filter(|entry| entry.active)
            .map(|entry| (entry.name.clone(), entry.value.clone()))
            .collect::<Vec<(String, String)>>()
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

#[cfg(test)]
mod tests {
    use srtemplate::SrTemplate;

    use crate::entities::{KeyValue, KeyValueTable};

    #[test]
    fn test_key_value_table_render() {
        let headers = vec![
            ("Content-Type", "application/json").into(),
            ("Authorization", "Bearer {{TOKEN}}").into(),
        ];
        let table = KeyValueTable::new(&headers);

        let context = SrTemplate::default();
        context.add_variable("TOKEN", &"Token_Value");

        let Ok(rendered) = table.render(&context) else {
            panic!("not ok");
        };
        assert_eq!(
            rendered.header("authorization"),
            Some(vec!["Bearer Token_Value"])
        );
    }

    #[test]
    fn test_key_value_render_fails() {
        let headers = vec![
            ("Content-Type", "application/json").into(),
            ("Authorization", "Bearer {{TOKEN}}").into(),
        ];
        let table = KeyValueTable::new(&headers);

        let context = SrTemplate::default();
        let rendered = table.render(&context);
        assert!(rendered.is_err());
    }

    #[test]
    fn test_key_value_active_pairs() {
        let headers = vec![
            KeyValue {
                name: "Accept".into(),
                value: "application/json".into(),
                active: false,
                secret: false,
            },
            KeyValue {
                name: "Content-Type".into(),
                value: "application/json".into(),
                active: true,
                secret: false,
            },
            KeyValue {
                name: "Authorization".into(),
                value: "Bearer roarrr".into(),
                active: false,
                secret: false,
            },
            KeyValue {
                name: "User-Agent".into(),
                value: "Cartero/0.1".into(),
                active: true,
                secret: false,
            },
        ];
        let table = KeyValueTable::new(&headers);

        let pairs = table.to_active_pairs();
        assert_eq!(
            vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("User-Agent".to_string(), "Cartero/0.1".to_string()),
            ],
            pairs
        );
    }

    #[test]
    fn test_key_value_table_header() {
        let headers = vec![
            ("Content-Type", "application/json").into(),
            ("Set-Cookie", "cookie2=value2").into(),
            ("Set-Cookie", "cookie1=value1").into(),
        ];
        let table = KeyValueTable::new(&headers);

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
        let table = KeyValueTable::new(&headers);

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
