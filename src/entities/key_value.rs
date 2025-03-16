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
