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
pub(crate) enum FileValue {
    Simple(String),
    Complex { value: String, active: bool },
}

impl ToField for FileValue {
    fn to_field(&self, key: &str) -> Field {
        match self {
            FileValue::Simple(str) => Object::builder()
                .property("key", key)
                .property("value", str)
                .property("active", true)
                .build(),
            FileValue::Complex { value, active } => Object::builder()
                .property("key", key)
                .property("value", value.clone())
                .property("active", *active)
                .build(),
        }
    }
}

impl From<Field> for FileValue {
    fn from(value: Field) -> Self {
        if value.active() && !value.masked() {
            FileValue::Simple(value.value().clone())
        } else {
            FileValue::Complex {
                value: value.value().to_string(),
                active: value.active(),
            }
        }
    }
}

impl Default for FileValue {
    fn default() -> Self {
        Self::Simple(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_field_simple() {
        let simple = FileValue::Simple("file.jpg".to_string());
        let convert = simple.to_field("upload");
        assert_eq!(convert.key(), "upload");
        assert_eq!(convert.value(), "file.jpg");
        assert!(convert.active());
    }

    #[test]
    fn test_to_field_complex() {
        let complex = FileValue::Complex {
            value: "file.jpg".into(),
            active: false,
        };
        let convert = complex.to_field("upload");
        assert_eq!(convert.key(), "upload");
        assert_eq!(convert.value(), "file.jpg");
        assert!(!convert.active());
    }

    #[test]
    fn test_from_field() {
        {
            let field: Field = Object::builder()
                .property("key", "upload")
                .property("value", "file.jpg")
                .property("active", false)
                .build();
            let serial = FileValue::from(field);
            if let FileValue::Complex { value, active } = serial {
                assert_eq!(value, "file.jpg");
                assert!(!active);
            } else {
                panic!("Expected Complex!");
            }
        }

        {
            let field: Field = Object::builder()
                .property("key", "upload")
                .property("value", "file.jpg")
                .property("active", false)
                .build();
            let serial = FileValue::from(field);
            if let FileValue::Complex { value, active } = serial {
                assert_eq!(value, "file.jpg");
                assert!(!active);
            } else {
                panic!("Expected Complex!");
            }
        }
    }

    #[test]
    fn test_from_field_simple() {
        let field: Field = Object::builder()
            .property("key", "upload")
            .property("value", "file.jpg")
            .property("active", true)
            .build();
        let serial = FileValue::from(field);
        if let FileValue::Simple(value) = serial {
            assert_eq!(value, "file.jpg");
        } else {
            panic!("Expected Simple!");
        }
    }
}
