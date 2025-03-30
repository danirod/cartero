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

use std::collections::HashMap;

use gio::prelude::{ListModelExt, ListModelExtManual};
use glib::subclass::prelude::*;
use glib::{prelude::*, Object};

use crate::field::Field;

glib::wrapper! {
    pub struct FieldTable(ObjectSubclass<imp::FieldTable>) @implements gio::ListModel;
}

impl Default for FieldTable {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl FieldTable {
    pub fn group_by_key(&self) -> HashMap<String, Vec<Field>> {
        self.iter::<Field>().fold(HashMap::new(), |mut map, item| {
            if let Ok(field) = item {
                match map.get_mut(field.key().as_str()) {
                    None => {
                        map.insert(field.key().to_string(), vec![field.clone()]);
                    }
                    Some(old) => old.push(field.clone()),
                };
            }
            map
        })
    }

    pub fn insert(&self, field: &Field) {
        {
            let mut fields = self.imp().fields.borrow_mut();
            fields.push(field.clone());
        }
        let len = { self.imp().fields.borrow().len() };
        self.items_changed(len as u32, 0, 1);
    }

    pub fn remove(&self, pos: u32) {
        {
            let mut fields = self.imp().fields.borrow_mut();
            // Panics in case of out of bounds.
            fields.remove(pos as usize);
        }
        // If we reach here, we survived delete.
        self.items_changed(pos, 1, 0);
    }
}

impl<'a, T> From<T> for FieldTable
where
    T: IntoIterator<Item = &'a Field>,
{
    fn from(iter: T) -> Self {
        let table: Self = Object::builder().build();
        table
            .imp()
            .fields
            .replace(iter.into_iter().cloned().collect());
        table
    }
}

mod imp {
    use gio::subclass::prelude::ListModelImpl;

    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct FieldTable {
        pub(super) fields: RefCell<Vec<Field>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldTable {
        const NAME: &'static str = "CarteroFieldTable";
        type Type = super::FieldTable;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for FieldTable {}

    impl ListModelImpl for FieldTable {
        fn item_type(&self) -> glib::Type {
            Field::static_type()
        }

        fn n_items(&self) -> u32 {
            self.fields.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            let fields = self.fields.borrow();
            fields.get(position as usize).map(|f| f.clone().upcast())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    pub fn test_valid_builder() {
        let table: FieldTable = Object::builder().build();
        assert_eq!(table.n_items(), 0);
    }

    #[test]
    pub fn test_insert_get_remove() {
        let field: Field = Object::builder()
            .property("key", "User-Agent")
            .property("value", "Mozilla/5.0")
            .build();
        let field2: Field = Object::builder()
            .property("key", "Content-Type")
            .property("value", "text/html")
            .build();
        let table: FieldTable = Object::builder().build();
        let inserts = Arc::new(Mutex::new(Vec::new()));
        let local_inserts = inserts.clone();
        table.connect_items_changed(move |_, pos, removed, added| {
            let mut vector = local_inserts.lock().unwrap();
            vector.push((pos, removed, added));
        });

        {
            table.insert(&field);
            assert_eq!(table.n_items(), 1);
            let (pos, removed, added) = inserts.lock().unwrap().last().unwrap().to_owned();
            assert_eq!(pos, 1);
            assert_eq!(removed, 0);
            assert_eq!(added, 1);
        }

        {
            table.insert(&field2);
            assert_eq!(table.n_items(), 2);
            let (pos, removed, added) = inserts.lock().unwrap().last().unwrap().to_owned();
            assert_eq!(pos, 2);
            assert_eq!(removed, 0);
            assert_eq!(added, 1);
        }

        {
            assert!(table
                .item(0)
                .is_some_and(|f| f.downcast::<Field>().is_ok_and(|f| f == field)));
            assert!(table
                .item(1)
                .is_some_and(|f| f.downcast::<Field>().is_ok_and(|f| f == field2)));
            assert!(table.item(2).is_none());
        }

        {
            table.remove(0);
            assert_eq!(table.n_items(), 1);
            let (pos, removed, added) = inserts.lock().unwrap().last().unwrap().to_owned();
            assert_eq!(pos, 0);
            assert_eq!(removed, 1);
            assert_eq!(added, 0);
        }

        {
            assert!(table
                .item(0)
                .is_some_and(|f| f.downcast::<Field>().is_ok_and(|f| f == field2)));
            assert!(table.item(1).is_none());
        }
    }

    #[test]
    fn test_field_table_from_fields() {
        let field: Field = Object::builder()
            .property("key", "User-Agent")
            .property("value", "Mozilla/5.0")
            .build();
        let field2: Field = Object::builder()
            .property("key", "Content-Type")
            .property("value", "text/html")
            .build();
        let fields = vec![field, field2];
        let table = FieldTable::from(&fields);
        assert_eq!(2, table.n_items());
        assert_eq!(Some(&fields[0]), table.item(0).and_downcast_ref::<Field>());
        assert_eq!(Some(&fields[1]), table.item(1).and_downcast_ref::<Field>());
    }

    #[test]
    fn test_field_table_group_by_key() {
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
        let table = FieldTable::from(&fields);

        let group = table.group_by_key();
        assert_eq!(1, group["Content-Type"].len());
        assert_eq!(2, group["Cookie"].len());
        assert_eq!(fields[1], group["Content-Type"][0]);
        assert_eq!(fields[0], group["Cookie"][0]);
        assert_eq!(fields[2], group["Cookie"][1]);
    }
}
