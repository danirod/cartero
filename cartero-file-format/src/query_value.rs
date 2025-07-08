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

use cartero_objects::Field;
use glib::Object;
use serde::{Deserialize, Serialize};

use crate::ToField;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum QueryValue {
    Simple(String),
    Complex { value: String, secret: bool },
}

impl ToField for QueryValue {
    fn to_field(&self, key: &str) -> Field {
        match self {
            Self::Simple(value) => Object::builder()
                .property("key", key)
                .property("value", value)
                .property("active", false)
                .property("masked", false)
                .build(),
            Self::Complex { value, secret } => Object::builder()
                .property("key", key)
                .property("value", value)
                .property("active", false)
                .property("masked", secret)
                .build(),
        }
    }
}

impl From<Field> for QueryValue {
    fn from(value: Field) -> Self {
        if value.masked() {
            Self::Complex {
                value: value.value().clone(),
                secret: value.masked(),
            }
        } else {
            Self::Simple(value.value().clone())
        }
    }
}

impl Default for QueryValue {
    fn default() -> Self {
        Self::Simple(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_field_simple() {
        let value = QueryValue::Simple("1234".into());
        let field = value.to_field("user_id");
        assert_eq!(field.key(), "user_id");
        assert_eq!(field.value(), "1234");
        assert!(!field.active());
        assert!(!field.masked());
    }

    #[test]
    fn test_to_field_complex() {
        let value = QueryValue::Complex {
            value: "1234".into(),
            secret: true,
        };
        let field = value.to_field("password");
        assert_eq!(field.key(), "password");
        assert_eq!(field.value(), "1234");
        assert!(!field.active());
        assert!(field.masked());
    }

    #[test]
    fn test_from_field_when_masked() {
        let field: Field = Object::builder()
            .property("key", "password")
            .property("value", "1234")
            .property("masked", true)
            .property("active", false)
            .build();
        let serial = QueryValue::from(field);
        match serial {
            QueryValue::Simple(_) => panic!("Expected QueryValue::Complex"),
            QueryValue::Complex { value, secret } => {
                assert_eq!(value, "1234");
                assert!(secret);
            }
        };
    }

    #[test]
    fn test_from_field_when_not_masked() {
        let field: Field = Object::builder()
            .property("key", "user_id")
            .property("value", "1234")
            .property("masked", false)
            .property("active", false)
            .build();
        let serial = QueryValue::from(field);
        match serial {
            QueryValue::Simple(value) => {
                assert_eq!(value, "1234");
            }
            QueryValue::Complex {
                value: _,
                secret: _,
            } => {
                panic!("Expected QueryValue::Simple");
            }
        };
    }
}
