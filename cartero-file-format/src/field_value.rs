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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub(crate) enum FieldValue {
    Simple(String),
    Complex {
        value: String,
        active: bool,
        secret: bool,
    },
}

impl ToField for FieldValue {
    fn to_field(&self, key: &str) -> Field {
        match self {
            FieldValue::Simple(str) => Object::builder()
                .property("key", key)
                .property("value", str)
                .property("active", true)
                .property("masked", false)
                .build(),
            FieldValue::Complex {
                value,
                active,
                secret,
            } => Object::builder()
                .property("key", key)
                .property("value", value.clone())
                .property("active", *active)
                .property("masked", *secret)
                .build(),
        }
    }
}

impl From<Field> for FieldValue {
    fn from(value: Field) -> Self {
        if value.active() && !value.masked() {
            FieldValue::Simple(value.value().clone())
        } else {
            FieldValue::Complex {
                value: value.value().to_string(),
                active: value.active(),
                secret: value.masked(),
            }
        }
    }
}

impl Default for FieldValue {
    fn default() -> Self {
        Self::Simple(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_field_simple() {
        let simple = FieldValue::Simple("text/html".to_string());
        let convert = simple.to_field("Content-Type");
        assert_eq!(convert.key(), "Content-Type");
        assert_eq!(convert.value(), "text/html");
        assert!(convert.active());
        assert!(!convert.masked());
    }

    #[test]
    fn test_to_field_complex() {
        let complex = FieldValue::Complex {
            value: "Bearer 1234".into(),
            active: false,
            secret: true,
        };
        let convert = complex.to_field("Authorization");
        assert_eq!(convert.key(), "Authorization");
        assert_eq!(convert.value(), "Bearer 1234");
        assert!(!convert.active());
        assert!(convert.masked());
    }

    #[test]
    fn test_from_field() {
        {
            let field: Field = Object::builder()
                .property("key", "Content-Type")
                .property("value", "text/html")
                .property("active", false)
                .property("masked", true)
                .build();
            let serial = FieldValue::from(field);
            if let FieldValue::Complex {
                value,
                active,
                secret,
            } = serial
            {
                assert_eq!(value, "text/html");
                assert!(!active);
                assert!(secret);
            } else {
                panic!("Expected Complex!");
            }
        }

        {
            let field: Field = Object::builder()
                .property("key", "Content-Type")
                .property("value", "text/html")
                .property("active", true)
                .property("masked", true)
                .build();
            let serial = FieldValue::from(field);
            if let FieldValue::Complex {
                value,
                active,
                secret,
            } = serial
            {
                assert_eq!(value, "text/html");
                assert!(active);
                assert!(secret);
            } else {
                panic!("Expected Complex!");
            }
        }

        {
            let field: Field = Object::builder()
                .property("key", "Content-Type")
                .property("value", "text/html")
                .property("active", false)
                .property("masked", false)
                .build();
            let serial = FieldValue::from(field);
            if let FieldValue::Complex {
                value,
                active,
                secret,
            } = serial
            {
                assert_eq!(value, "text/html");
                assert!(!active);
                assert!(!secret);
            } else {
                panic!("Expected Complex!");
            }
        }
    }

    #[test]
    fn test_from_field_simple() {
        let field: Field = Object::builder()
            .property("key", "Content-Type")
            .property("value", "text/html")
            .property("active", true)
            .property("masked", false)
            .build();
        let serial = FieldValue::from(field);
        if let FieldValue::Simple(value) = serial {
            assert_eq!(value, "text/html");
        } else {
            panic!("Expected Simple!");
        }
    }
}
