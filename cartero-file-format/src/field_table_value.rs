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

use std::{collections::HashMap, ops::Deref};

use cartero_objects::{Field, FieldTable};
use serde::{Deserialize, Serialize};

use crate::{field_value::FieldValue, ToField};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub(crate) enum FieldValueTableRow<T>
where
    T: From<Field> + ToField,
{
    Unique(T),
    Multiple(Vec<T>),
}

impl<T> FieldValueTableRow<T>
where
    T: From<Field> + ToField,
{
    pub fn to_fields(&self, key: &str) -> Vec<Field> {
        match self {
            Self::Unique(value) => vec![value.to_field(key)],
            Self::Multiple(values) => values.iter().map(|row| row.to_field(key)).collect(),
        }
    }
}

impl<T> From<Vec<Field>> for FieldValueTableRow<T>
where
    T: From<Field> + ToField,
{
    fn from(value: Vec<Field>) -> Self {
        if value.len() == 1 {
            Self::Unique(value[0].clone().into())
        } else {
            Self::Multiple(value.into_iter().map(T::from).collect())
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FieldTableValue<T>(HashMap<String, FieldValueTableRow<T>>)
where
    T: From<Field> + ToField;

impl<T> FieldTableValue<T>
where
    T: From<Field> + ToField,
{
    pub(crate) fn new(map: HashMap<String, FieldValueTableRow<T>>) -> Self {
        Self(map)
    }
}

impl<T> Deref for FieldTableValue<T>
where
    T: From<Field> + ToField,
{
    type Target = HashMap<String, FieldValueTableRow<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<FieldTable> for FieldTableValue<T>
where
    T: From<Field> + ToField,
{
    fn from(value: FieldTable) -> Self {
        let rows = value.group_by_key();
        let inner = rows
            .into_iter()
            .map(|(key, rows)| (key, FieldValueTableRow::from(rows)))
            .collect();
        Self(inner)
    }
}

impl<T> From<FieldTableValue<T>> for FieldTable
where
    T: From<Field> + ToField,
{
    fn from(value: FieldTableValue<T>) -> Self {
        let fields: Vec<Field> = value
            .0
            .iter()
            .flat_map(|(key, values)| values.to_fields(key))
            .collect();
        Self::from_iter(fields)
    }
}

#[cfg(test)]
mod tests {
    use gio::prelude::{ListModelExt, ListModelExtManual};
    use glib::Object;

    use crate::field_value::FieldValue;

    use super::*;

    #[test]
    pub fn test_table_row_unique_to_fields() {
        let value = FieldValue::Simple("text/html".into());
        let row = FieldValueTableRow::Unique(value);
        let result = row.to_fields("Content-Type");
        assert_eq!(1, result.len());
        assert_eq!("Content-Type", result[0].key());
        assert_eq!("text/html", result[0].value());
        assert!(result[0].active());
        assert!(!result[0].masked());
    }

    #[test]
    pub fn test_table_row_multiple_to_fields() {
        let value1 = FieldValue::Simple("user_id=1234".into());
        let value2 = FieldValue::Simple("token=asdf".into());
        let row = FieldValueTableRow::Multiple(vec![value1, value2]);
        let result = row.to_fields("Cookie");
        assert_eq!(2, result.len());
        assert_eq!("Cookie", result[0].key());
        assert_eq!("user_id=1234", result[0].value());
        assert!(result[0].active());
        assert!(!result[0].masked());
        assert_eq!("Cookie", result[1].key());
        assert_eq!("token=asdf", result[1].value());
        assert!(result[1].active());
        assert!(!result[1].masked());
    }

    #[test]
    pub fn test_field_table_to_value() {
        let field: Field = Object::builder()
            .property("key", "Cookie")
            .property("value", "admin=1234")
            .build();
        let field2: Field = Object::builder()
            .property("key", "Content-Type")
            .property("value", "text/html")
            .build();
        let field3: Field = Object::builder()
            .property("key", "Cookie")
            .property("value", "session=2345")
            .build();
        let fields = vec![field, field2, field3];
        let table = FieldTable::from_iter(fields);

        let value: FieldTableValue<FieldValue> = FieldTableValue::from(table);
        assert_eq!(2, value.0.len());

        let cookies = &value.0["Cookie"];
        let content_type = &value.0["Content-Type"];

        let FieldValueTableRow::Unique(value) = content_type else {
            panic!("Content-Type not unique!");
        };
        assert_eq!(FieldValue::Simple("text/html".into()), value.clone());

        let FieldValueTableRow::Multiple(values) = cookies else {
            panic!("Cookies unique!");
        };
        assert_eq!(2, values.len());
        assert_eq!(FieldValue::Simple("admin=1234".into()), values[0].clone());
        assert_eq!(FieldValue::Simple("session=2345".into()), values[1].clone());
    }

    #[test]
    pub fn test_field_table_from_value() {
        let cookies = FieldValueTableRow::Multiple(vec![
            FieldValue::Simple("admin=1234".into()),
            FieldValue::Simple("session=2345".into()),
        ]);
        let types = FieldValueTableRow::Unique(FieldValue::Simple("text/html".into()));
        let rows = HashMap::from([
            ("Content-Type".to_string(), types),
            ("Cookie".to_string(), cookies),
        ]);
        let table = FieldTableValue(rows);
        let value = FieldTable::from(table);
        assert_eq!(3, value.n_items());

        assert!(value
            .iter::<Field>()
            .any(|f| f.is_ok_and(|f| f.key() == "Content-Type" && f.value() == "text/html")));
        assert!(value
            .iter::<Field>()
            .any(|f| f.is_ok_and(|f| f.key() == "Cookie" && f.value() == "admin=1234")));
        assert!(value
            .iter::<Field>()
            .any(|f| f.is_ok_and(|f| f.key() == "Cookie" && f.value() == "session=2345")));
    }
}
