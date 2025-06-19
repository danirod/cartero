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

use gio::prelude::ListModelExt;
use glib::subclass::prelude::*;
use glib::{prelude::*, Object};

use crate::field::Field;

glib::wrapper! {
    /// A sorted table that groups multiple [fields][Field].
    ///
    /// Internally, a field table is treated like a list of fields, assigning
    /// a position to every item in the list. The table allows to collect
    /// fields and to iterate over them.
    ///
    /// ## Properties
    ///
    /// `FieldTable` currently does not expose properties. FieldTable implements
    /// `ListModel`, and thus data is accessed using the ListModel interface.
    ///
    /// ## Building a Table
    ///
    /// There are two ways:
    ///
    /// - Use the `default` function to create an empty table.
    ///
    /// ```
    /// use cartero_objects::{Field, FieldTable};
    ///
    /// let table = FieldTable::default();
    /// let field = Field::from(("User-Agent", "Mozilla/5.0"));
    /// table.insert(&field);
    /// ```
    ///
    /// - Use the `from_iter()` function to build a table from an existing
    ///   collection of fields.
    ///
    /// ```
    /// use cartero_objects::{Field, FieldTable};
    ///
    /// let field1 = Field::from(("user_id", "1000"));
    /// let field2 = Field::from(("category_id", "10"));
    /// let fields = vec![field1, field2];
    /// let table = FieldTable::from_iter(fields);
    /// ```
    ///
    /// Once you have a table, you may use the standard [Gio.ListModel]
    /// interface methods and signals to read the contents of the table, and
    /// use the impl methods of this object to change the inner elements of
    /// the table.
    ///
    /// [Gio.ListModel]: https://docs.gtk.org/gio/iface.ListModel.html
    pub struct FieldTable(ObjectSubclass<imp::FieldTable>) @implements gio::ListModel;
}

impl Default for FieldTable {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl FieldTable {
    /// Add a new field to the table.
    ///
    /// The given `field` is inserted at the bottom of the table, and
    /// assigned the highest position so far in the table. Emits an
    /// `items-changed` signal when done.
    pub fn insert(&self, field: &Field) {
        {
            let mut fields = self.imp().fields.borrow_mut();
            fields.push(field.clone());
        }
        let len = { self.imp().fields.borrow().len() };
        self.items_changed(len as u32, 0, 1);
    }

    pub fn field(&self, pos: u32) -> Option<Field> {
        self.item(pos).and_downcast::<Field>()
    }

    pub fn replace(&self, table: &Self) {
        let old_len = self.n_items();
        let new_len = table.n_items();

        {
            let mut fields = self.imp().fields.borrow_mut();
            let mut new_fields = { table.imp().fields.borrow().clone() };
            fields.clear();
            fields.append(&mut new_fields);
        }

        self.items_changed(0, old_len, new_len);
    }

    /// Remove a field from the table.
    ///
    /// The position of the element to remove has to be given as a parameter,
    /// and the field at that position will be yanked. **Will panic if the
    /// given index is out of bounds**. Emits an `items-changed` signal when
    /// done.
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

impl FromIterator<Field> for FieldTable {
    fn from_iter<T: IntoIterator<Item = Field>>(iter: T) -> Self {
        let table = Self::default();
        table
            .imp()
            .fields
            .replace(iter.into_iter().collect::<Vec<Field>>());
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
    use std::{
        collections::HashSet,
        sync::{Arc, Mutex},
    };

    use super::*;

    #[test]
    pub fn test_valid_builder() {
        let table: FieldTable = Object::builder().build();
        assert_eq!(table.n_items(), 0);
    }

    #[test]
    pub fn test_valid_from_iter_vec() {
        let field = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        let field2 = Field::builder()
            .key("Content-Type")
            .value("text/html")
            .build();
        let fields = vec![field, field2];
        let table = FieldTable::from_iter(fields);
        assert_eq!(2, table.n_items());
        assert_eq!("User-Agent", table.field(0).unwrap().key(),);
        assert_eq!("Content-Type", table.field(1).unwrap().key(),);
    }

    #[test]
    pub fn test_valid_from_iter_set() {
        let field = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        let field2 = Field::builder()
            .key("Content-Type")
            .value("text/html")
            .build();
        let mut fields = HashSet::new();
        fields.insert(field);
        fields.insert(field2);
        let table = FieldTable::from_iter(fields);
        assert_eq!(2, table.n_items());
    }

    #[test]
    pub fn test_insert_get_remove() {
        let field = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        let field2 = Field::builder()
            .key("Content-Type")
            .value("text/html")
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
            assert!(table.field(0).is_some_and(|f| f == field));
            assert!(table.field(1).is_some_and(|f| f == field2));
            assert!(table.field(2).is_none());
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
            assert!(table.field(0).is_some_and(|f| f == field2));
            assert!(table.item(1).is_none());
        }
    }

    #[test]
    pub fn test_replace_field_table() {
        let table1 = FieldTable::from_iter(vec![
            Field::builder()
                .key("User-Agent")
                .value("Mozilla/5.0")
                .build(),
            Field::builder()
                .key("Accept")
                .value("application/json")
                .build(),
        ]);
        let table2 = FieldTable::from_iter(vec![
            Field::builder()
                .key("Content-Type")
                .value("text/html")
                .build(),
            Field::builder().key("Host").value("example.com").build(),
            Field::builder().key("Server").value("nginx/1.0").build(),
        ]);

        assert_eq!(2, table1.n_items());
        assert_eq!("User-Agent", table1.field(0).unwrap().key());
        assert_eq!("Accept", table1.field(1).unwrap().key());

        table1.replace(&table2);

        assert_eq!(3, table1.n_items());
        assert_eq!("Content-Type", table1.field(0).unwrap().key());
        assert_eq!("Host", table1.field(1).unwrap().key());
        assert_eq!("Server", table1.field(2).unwrap().key());
    }
}
