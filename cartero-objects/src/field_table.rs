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
use itertools::{EitherOrBoth, Itertools};
use srtemplate::SrTemplate;

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
    /// Register the change signal for this field, so that whenever the inner
    /// Field changes because it emits a "change" signal, this table broadcasts
    /// the same event upwards.
    fn connect_signal(&self, field: &Field, n_item: usize) {
        let mut signals = self.imp().signals.borrow_mut();
        let signal = field.connect_closure(
            "changed",
            false,
            glib::closure_local!(
                #[weak(rename_to = field_table)]
                self,
                move |_field: Field, param: &str| {
                    let parameter = format!("[{}].{}", n_item, param);
                    field_table.emit_by_name::<()>("changed", &[&parameter]);
                }
            ),
        );
        signals.insert(field.clone(), Some(signal));
    }

    /// Disconnects the previously added signal handler for this field, so
    /// that future "change" events triggered in the field are not broadcasted
    /// upwards from this table.
    fn disconnect_signal(&self, field: &Field) {
        let mut signals = self.imp().signals.borrow_mut();
        if let Some(handler) = signals.remove(field) {
            if let Some(signal) = handler {
                field.disconnect(signal);
            }
        }
    }

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
        self.connect_signal(field, len - 1);
        self.items_changed(len as u32, 0, 1);
        self.emit_by_name::<()>("changed", &[&""]);
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

            fields
                .iter()
                .for_each(|field| self.disconnect_signal(field));

            fields.clear();
            fields.append(&mut new_fields);

            new_fields.iter().enumerate().for_each(|(pos, field)| {
                self.connect_signal(field, pos);
            });
        }

        self.items_changed(0, old_len, new_len);
        self.emit_by_name::<()>("changed", &[&""]);
    }

    /// Remove a field from the table.
    ///
    /// The position of the element to remove has to be given as a parameter,
    /// and the field at that position will be yanked. **Will panic if the
    /// given index is out of bounds**. Emits an `items-changed` signal when
    /// done.
    pub fn remove(&self, pos: u32) {
        if let Some(field) = self.field(pos) {
            self.disconnect_signal(&field);
        }

        {
            let mut fields = self.imp().fields.borrow_mut();
            // Panics in case of out of bounds.
            fields.remove(pos as usize);
        }
        // If we reach here, we survived delete.
        self.items_changed(pos, 1, 0);
        self.emit_by_name::<()>("changed", &[&""]);
    }

    /// Groups by key every field contained in this table, accepting duplicates.
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

    pub fn render(&self, template: &SrTemplate) -> Result<Self, srtemplate::Error> {
        self.iter::<Field>()
            .filter_map(|r| r.ok())
            .map(|field| {
                let key = template.render(field.key())?;
                let value = template.render(field.value())?;
                Ok(Field::builder()
                    .key(key)
                    .value(value)
                    .active(field.active())
                    .masked(field.masked())
                    .build())
            })
            .collect::<Result<Self, srtemplate::Error>>()
    }

    pub fn template_processor(&self) -> SrTemplate<'static> {
        self.iter::<Field>()
            .filter_map(|r| r.ok())
            .filter(|field| field.active())
            .fold(SrTemplate::default(), |acc, field| {
                acc.add_variable(field.key(), field.value());
                acc
            })
    }

    /// Apply a mass update to the keys and values of this FieldTable using the source keys and
    /// values given in the slice. This is done as effectively as possible, instantating the less
    /// amount of new Field objects as it can, so it is appropiate for real time updates, such as
    /// the ones required to sync the URL entry of Cartero with the query parameter table.
    ///
    /// The conditions to use this function are:
    ///
    /// - Only Fields in the FieldTable that are active will be updated. Inactive fields will be
    ///   ignored and let at their original position. Updates will just happen with the next
    ///   element of the FieldTable that is active.
    /// - If the given slice has more elements than the FieldTable, new Fields are added to the
    ///   table, to accomodate them.
    /// - If the given slice has less elements than the FieldTable, any Field that cannot be mapped
    ///   to an input slice will be set as inactive.
    ///
    /// Make sure that you know what you're doing when calling this method.
    pub fn reconcile<K, V>(&self, entries: &[(K, V)])
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let active_field_iter = self
            .iter::<Field>()
            .filter_map(|f| f.ok())
            .filter(|f| f.active());
        let source_iter = entries.iter();
        let mut new_items = Vec::new();

        for group in active_field_iter.zip_longest(source_iter) {
            match group {
                EitherOrBoth::Both(field, (key, value)) => {
                    // Update the data.
                    field.set_key(key.as_ref());
                    field.set_value(value.as_ref());
                }
                EitherOrBoth::Left(field) => {
                    // This field doesn't match to anything, so it will be disabled.
                    field.set_active(false);
                }
                EitherOrBoth::Right((key, value)) => {
                    let field = Field::builder()
                        .key(key.as_ref())
                        .value(value.as_ref())
                        .build();
                    new_items.push(field);
                }
            }
        }
        for new_item in new_items {
            self.insert(&new_item);
        }
    }
}

impl FromIterator<Field> for FieldTable {
    fn from_iter<T: IntoIterator<Item = Field>>(iter: T) -> Self {
        let table = Self::default();
        table
            .imp()
            .fields
            .replace(iter.into_iter().collect::<Vec<Field>>());

        // Manually initialize the events.
        {
            let fields = table.imp().fields.borrow();
            fields.iter().enumerate().for_each(|(pos, field)| {
                table.connect_signal(field, pos);
            });
        }

        table
    }
}

mod imp {
    use gio::subclass::prelude::ListModelImpl;
    use glib::{subclass::Signal, SignalHandlerId};

    use super::*;
    use std::{cell::RefCell, sync::OnceLock};

    #[derive(Default)]
    pub struct FieldTable {
        pub(super) fields: RefCell<Vec<Field>>,

        pub(super) signals: RefCell<HashMap<Field, Option<SignalHandlerId>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldTable {
        const NAME: &'static str = "CarteroFieldTable";
        type Type = super::FieldTable;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for FieldTable {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("changed")
                    .param_types([String::static_type()])
                    .build()]
            })
        }
    }

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

    use crate::utils::test::{assert_emits_signal, assert_not_emits_signal};

    use super::*;

    #[test]
    pub fn test_default() {
        let table = FieldTable::default();
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
    fn test_group_by_key() {
        let fields = vec![
            Field::from(("category_id", "10")),
            Field::from(("tag_id", "20")),
            Field::from(("tag_id", "30")),
        ];
        let table = FieldTable::from_iter(fields);

        let group = table.group_by_key();
        assert_eq!(group.len(), 2);

        let category_id = group.get("category_id").unwrap();
        assert_eq!(category_id.len(), 1);
        assert_eq!(category_id[0].value(), "10");

        let tag_id = group.get("tag_id").unwrap();
        assert_eq!(tag_id.len(), 2);
        if tag_id[0].value() == "20" {
            assert_eq!(tag_id[1].value(), "30");
        } else if tag_id[0].value() == "30" {
            assert_eq!(tag_id[1].value(), "20");
        } else {
            panic!("invalid elements in the tag");
        }

        assert!(group.get("empty").is_none());
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

    #[test]
    fn test_template_processor() {
        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("API_ROOT")
                .value("http://localhost:8000")
                .build(),
            Field::builder().key("TOKEN").value("12341234").build(),
        ]);

        let processor = table.template_processor();
        assert!(processor.contains_variable("API_ROOT"));
        assert!(processor.contains_variable("TOKEN"));
        assert_eq!(
            processor.render("{{API_ROOT}}/api/v1").unwrap(),
            "http://localhost:8000/api/v1"
        );
        assert!(processor.render("{{INVALID}}").is_err());
    }

    #[test]
    fn test_template_processor_is_immutable() {
        let field = Field::builder()
            .key("API_ROOT")
            .value("http://localhost:3000")
            .build();
        let table = FieldTable::from_iter(vec![field.clone()]);

        // The processor renders the current state of the field table
        let processor = table.template_processor();
        assert_eq!(
            processor.render("{{ API_ROOT }}/api/v1").unwrap(),
            "http://localhost:3000/api/v1"
        );

        // Just because you update a value, it won't update the processor
        field.set_value("http://api.example.com");
        assert_eq!(
            processor.render("{{ API_ROOT }}/api/v1").unwrap(),
            "http://localhost:3000/api/v1"
        );

        // Of course if you create a new processor, the value is updated
        let processor = table.template_processor();
        assert_eq!(
            processor.render("{{ API_ROOT }}/api/v1").unwrap(),
            "http://api.example.com/api/v1"
        );
    }

    #[test]
    fn test_template_processor_can_survive_the_field_table() {
        let processor = {
            let field = Field::builder()
                .key("API_ROOT")
                .value("http://localhost:3000")
                .build();
            let table = FieldTable::from_iter(vec![field.clone()]);
            table.template_processor()
        };
        assert!(processor.contains_variable("API_ROOT"));
    }

    #[test]
    fn test_template_processor_deactivated_variable() {
        let table = FieldTable::from_iter(vec![Field::builder()
            .key("API_ROOT")
            .value("http://localhost:8000")
            .active(false)
            .build()]);

        let processor = table.template_processor();
        assert!(!processor.contains_variable("API_ROOT"));
    }

    #[test]
    fn test_template_processor_overriding_variable() {
        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("API_ROOT")
                .value("http://localhost:3000")
                .build(),
            Field::builder()
                .key("API_ROOT")
                .value("https://api.example.com")
                .build(),
        ]);

        let processor = table.template_processor();
        assert_eq!(
            processor.render("{{ API_ROOT }}").unwrap(),
            "https://api.example.com"
        );
    }

    #[test]
    fn test_template_processor_overriding_variable_is_disabled() {
        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("API_ROOT")
                .value("http://localhost:3000")
                .build(),
            Field::builder()
                .key("API_ROOT")
                .value("https://api.example.com")
                .active(false)
                .build(),
        ]);

        let processor = table.template_processor();
        assert_eq!(
            processor.render("{{ API_ROOT }}").unwrap(),
            "http://localhost:3000"
        );
    }

    #[test]
    fn test_render_with_valid_variables() {
        let template = SrTemplate::default();
        template.add_variable("API_ROOT", "http://localhost:3000");
        template.add_variable("API_KEY", "12341234");
        template.add_variable("KEY", "category");

        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("Location")
                .value("{{ API_ROOT }}/v1/users")
                .build(),
            Field::builder()
                .key("{{ KEY }}")
                .value("10")
                .active(false)
                .build(),
            Field::builder()
                .key("Authorization")
                .value("Bearer {{ API_KEY }}")
                .masked(true)
                .build(),
            Field::builder()
                .key("X-Api-Key")
                .value("{{API_KEY}}")
                .active(false)
                .masked(true)
                .build(),
        ]);

        let render_table = table.render(&template).unwrap();
        assert_eq!(render_table.n_items(), 4);

        let values = render_table.group_by_key();

        assert_field(
            &values["Location"][0],
            "Location",
            "http://localhost:3000/v1/users",
            true,
            false,
        );
        assert_field(
            &values["Authorization"][0],
            "Authorization",
            "Bearer 12341234",
            true,
            true,
        );
        assert_field(
            &values["X-Api-Key"][0],
            "X-Api-Key",
            "12341234",
            false,
            true,
        );
        assert_field(&values["category"][0], "category", "10", false, false);
    }

    #[test]
    fn test_render_with_invalid_variables() {
        let template = SrTemplate::default();

        let table = FieldTable::from_iter(vec![Field::builder()
            .key("Location")
            .value("{{ API_ROOT }}/v1/users")
            .build()]);
        let render_table = table.render(&template);
        let Err(srtemplate::Error::VariableNotFound(var)) = render_table else {
            panic!("expected err");
        };
        assert_eq!(var, "API_ROOT");
    }

    #[test]
    fn test_reconcile_simple_update() {
        let table = FieldTable::from_iter(vec![
            Field::builder().key("cat_id").value("10").build(),
            Field::builder().key("limit").value("20").build(),
        ]);
        let update = [("cat_id", "15"), ("offset", "50")];
        table.reconcile(&update);
        assert_field(&table.field(0).unwrap(), "cat_id", "15", true, false);
        assert_field(&table.field(1).unwrap(), "offset", "50", true, false);
    }

    #[test]
    fn test_reconcile_ignores_inactive() {
        let table = FieldTable::from_iter(vec![
            Field::builder().key("cat_id").value("10").build(),
            Field::builder()
                .key("offset")
                .value("5")
                .active(false)
                .build(),
            Field::builder().key("limit").value("20").build(),
        ]);
        let update = [("cat_id", "15"), ("sort", "-updated")];
        table.reconcile(&update);
        assert_field(&table.field(0).unwrap(), "cat_id", "15", true, false);
        assert_field(&table.field(1).unwrap(), "offset", "5", false, false);
        assert_field(&table.field(2).unwrap(), "sort", "-updated", true, false);
    }

    #[test]
    fn test_reconcile_adds_new_fields() {
        let table = FieldTable::from_iter(vec![
            Field::builder().key("cat_id").value("10").build(),
            Field::builder().key("limit").value("20").build(),
        ]);
        let update = [("cat_id", "15"), ("limit", "25"), ("sort", "created")];
        table.reconcile(&update);
        assert_field(&table.field(0).unwrap(), "cat_id", "15", true, false);
        assert_field(&table.field(1).unwrap(), "limit", "25", true, false);
        assert_field(&table.field(2).unwrap(), "sort", "created", true, false);
    }

    #[test]
    fn test_reconcile_marks_fields_as_disabled() {
        let table = FieldTable::from_iter(vec![
            Field::builder().key("cat_id").value("10").build(),
            Field::builder().key("limit").value("20").build(),
        ]);
        let update = [("cat_id", "15")];
        table.reconcile(&update);
        assert_field(&table.field(0).unwrap(), "cat_id", "15", true, false);
        assert_field(&table.field(1).unwrap(), "limit", "20", false, false);
    }

    #[test]
    fn test_emits_items_changed_when_item_is_added() {
        let table = FieldTable::default();
        assert_emits_signal(&table, "items-changed", || {
            table.insert(
                &Field::builder()
                    .key("user-agent")
                    .value("mozilla/5.0")
                    .build(),
            );
        });
        assert_not_emits_signal(&table, "items-changed", || {
            if let Some(field) = table.field(0) {
                field.set_value("internet explorer");
            }
        });
        assert_emits_signal(&table, "items-changed", || table.remove(0));
    }

    #[test]
    fn test_emits_changed_when_a_field_changes() {
        let table = FieldTable::default();
        assert_emits_signal(&table, "changed", || {
            table.insert(
                &Field::builder()
                    .key("user-agent")
                    .value("mozilla/5.0")
                    .build(),
            );
        });
        assert_emits_signal(&table, "changed", || {
            if let Some(field) = table.field(0) {
                field.set_value("internet explorer");
            }
        });
        assert_emits_signal(&table, "changed", || table.remove(0));
    }

    fn assert_field(f: &Field, key: &str, value: &str, active: bool, masked: bool) {
        assert_eq!(f.key(), key);
        assert_eq!(f.value(), value);
        assert_eq!(f.active(), active);
        assert_eq!(f.masked(), masked);
    }
}
