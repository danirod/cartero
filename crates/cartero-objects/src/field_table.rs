// Copyright 2024-2026 the Cartero authors
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
use glib::{Object, prelude::*};
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

pub enum CombinePriority {
    Prepend,
    Append,
}

impl FieldTable {
    /// Will merge the contents of the given FieldTable into this FieldTable.
    /// Depending on the priority given, the elements will be prepended or
    /// appended. This is important because it affects how template processors
    /// will perceive the combined variables. If appended, they will have more
    /// priority if a variable has the same name.
    pub fn combine(&self, another: &Self, priority: CombinePriority) {
        for n in 0..another.n_items() {
            let next = another.field(n).expect("Empty but not empty?");
            match priority {
                CombinePriority::Prepend => self.insert_at(&next, n),
                CombinePriority::Append => self.insert(&next),
            }
        }
    }

    pub fn dup(&self) -> Self {
        let table = Self::default();
        for field in self.iter::<Field>() {
            if let Ok(field) = &field {
                table.insert(&field.dup());
            }
        }
        table
    }

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
        if let Some(handler) = signals.remove(field)
            && let Some(signal) = handler
        {
            field.disconnect(signal);
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
        self.items_changed((len - 1) as u32, 0, 1);
        let value = format!("[{}]", len - 1);
        self.emit_by_name::<()>("changed", &[&value]);
    }

    /// Inserts a field into the table so that it lands at the requested position.
    ///
    /// When the given `pos` value is the position of a current value in the
    /// FieldTable (so, when `pos` is a number in inclusive range 0 to n_items(),
    /// the new item is added at that position and everything is moved one position
    /// to the right.
    ///
    /// When the given `pos` is higher than the current n_items(), it behaves like
    /// `insert()`.
    pub fn insert_at(&self, field: &Field, pos: u32) {
        if pos >= self.n_items() {
            self.insert(field);
            return;
        }

        let upos = pos as usize;
        {
            let mut fields = self.imp().fields.borrow_mut();
            fields.insert(upos, field.clone());

            // Signals from elements that have shifted to the right need to change
            // because they are using the old index position.
            for i in (upos + 1)..fields.len() {
                self.disconnect_signal(&fields[i]);
            }

            // Signals from elements that have shifted to the right need to change
            // because they are using the old index position.
            for i in (upos)..fields.len() {
                self.connect_signal(&fields[i], i);
            }
        }

        self.items_changed(pos, 0, 1);
        let value = format!("[{}]", pos);
        self.emit_by_name::<()>("changed", &[&value]);
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

            fields.iter().enumerate().for_each(|(pos, field)| {
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

    /// Removes all the fields from the table.
    pub fn clear(&self) {
        while self.n_items() > 0 {
            self.remove(0);
        }
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

    fn find_by_name_(&self, name: &str, ignore_case: bool) -> Option<Vec<String>> {
        let compare_key = if ignore_case {
            name.to_lowercase()
        } else {
            name.to_owned()
        };

        let matches = self
            .iter::<Field>()
            .filter_map(|r| r.ok())
            .filter(|field| field.active())
            .filter_map(|field| {
                let qualifying_name = if ignore_case {
                    field.key().to_lowercase()
                } else {
                    field.key().to_owned()
                };
                if qualifying_name == compare_key {
                    Some(field.value())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    /// Finds active values by their name. Returns the values of active fields
    /// within the table with the given specific name. Note that if the field
    /// is not active, it is not considered. Capitalization matters. For
    /// HTTP headers, you should use the [find_by_name_icase] function.
    ///
    /// ```
    /// use cartero_objects::{Field, FieldTable};
    ///
    /// let field1 = Field::from(("Client-Id", "12341234"));
    /// let field2 = Field::from(("Api-Key", "101010"));
    /// let field3 = Field::from(("sort", "price"));
    /// let field4 = Field::from(("sort", "creation_date"));
    /// field2.set_active(false);
    ///
    /// let table = FieldTable::from_iter(vec![field1, field2, field3, field4]);
    ///
    /// // The value is returned.
    /// let client_id = table.find_by_name("Client-Id");
    /// assert_eq!(client_id.unwrap(), vec!["12341234"]);
    ///
    /// // The vector may have multiple elements.
    /// let sort = table.find_by_name("sort");
    /// assert_eq!(sort.unwrap(), vec!["price", "creation_date"]);
    ///
    /// // If the field is disabled, it is not included.
    /// let api_key = table.find_by_name("Api-Key");
    /// assert!(api_key.is_none());
    ///
    /// // The capitalization must match.
    /// let invalid_case = table.find_by_name("client-id");
    /// assert!(invalid_case.is_none());
    /// ```
    pub fn find_by_name(&self, name: &str) -> Option<Vec<String>> {
        self.find_by_name_(name, false)
    }

    /// Finds active values by their name, but ignoring case. Usually HTTP
    /// headers ignore capitalization and in HTTP/2.0 and above, they are
    /// always lowercase, but if you are working with HTTP headers, this is
    /// the method you are usually looking for.
    ///
    /// As is the case with [find_by_name], fields that are not active are not
    /// taken into account.
    ///
    /// ```
    /// use cartero_objects::{Field, FieldTable};
    ///
    /// let field1 = Field::from(("Client-Id", "12341234"));
    /// let field2 = Field::from(("Api-Key", "101010"));
    /// let field3 = Field::from(("sort", "price"));
    /// let field4 = Field::from(("sort", "creation_date"));
    /// field2.set_active(false);
    ///
    /// let table = FieldTable::from_iter(vec![field1, field2, field3, field4]);
    ///
    /// // The value is returned.
    /// let client_id = table.find_by_name_icase("Client-Id");
    /// assert_eq!(client_id.unwrap(), vec!["12341234"]);
    ///
    /// // The vector may have multiple elements.
    /// let sort = table.find_by_name_icase("sort");
    /// assert_eq!(sort.unwrap(), vec!["price", "creation_date"]);
    ///
    /// // If the field is disabled, it is not included.
    /// let api_key = table.find_by_name_icase("Api-Key");
    /// assert!(api_key.is_none());
    ///
    /// // This one won't care about capitalization
    /// let invalid_case = table.find_by_name_icase("client-id");
    /// assert!(invalid_case.is_some());
    /// ```
    pub fn find_by_name_icase(&self, name: &str) -> Option<Vec<String>> {
        self.find_by_name_(name, true)
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
        let mut items_to_delete = Vec::new();

        for group in active_field_iter.zip_longest(source_iter) {
            match group {
                EitherOrBoth::Both(field, (key, value)) => {
                    // Update the data.
                    field.set_key(key.as_ref());
                    field.set_value(value.as_ref());
                }
                EitherOrBoth::Left(field) => {
                    // This field doesn't match to anything, so it will be disabled.
                    items_to_delete.push(field.clone());
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
        for old_item in items_to_delete {
            let item_pos = self
                .iter::<Field>()
                .position(|f| f.is_ok_and(|f| f.eq(&old_item)));
            if let Some(pos) = item_pos {
                self.remove(pos as u32);
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
    use glib::{SignalHandlerId, subclass::Signal};

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
                vec![
                    Signal::builder("changed")
                        .param_types([String::static_type()])
                        .build(),
                ]
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
    fn combine_prepend() {
        let f1 = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        let f2 = Field::builder()
            .key("Content-Type")
            .value("text/html")
            .build();
        let f3 = Field::builder().key("Host").value("example.com").build();
        let ft1 = FieldTable::from_iter(vec![f1, f2]);
        let ft2 = FieldTable::from_iter(vec![f3]);
        assert_eq!(2, ft1.n_items());
        ft1.combine(&ft2, CombinePriority::Prepend);
        assert_eq!(3, ft1.n_items());
        assert_eq!("User-Agent", ft1.field(1).unwrap().key(),);
        assert_eq!("Content-Type", ft1.field(2).unwrap().key(),);
        assert_eq!("Host", ft1.field(0).unwrap().key(),);
    }

    #[test]
    fn combine_append() {
        let f1 = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        let f2 = Field::builder()
            .key("Content-Type")
            .value("text/html")
            .build();
        let f3 = Field::builder().key("Host").value("example.com").build();
        let ft1 = FieldTable::from_iter(vec![f1, f2]);
        let ft2 = FieldTable::from_iter(vec![f3]);
        assert_eq!(2, ft1.n_items());
        ft1.combine(&ft2, CombinePriority::Append);
        assert_eq!(3, ft1.n_items());
        assert_eq!("User-Agent", ft1.field(0).unwrap().key(),);
        assert_eq!("Content-Type", ft1.field(1).unwrap().key(),);
        assert_eq!("Host", ft1.field(2).unwrap().key(),);
    }

    #[test]
    fn combine_append_may_change_priorities() {
        let staging = Field::builder().key("ENVIRONMENT").value("staging").build();
        let production = Field::builder()
            .key("ENVIRONMENT")
            .value("production")
            .build();
        let table = FieldTable::from_iter(vec![staging]);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(rendered, Ok("staging".to_string()));

        let incoming = FieldTable::from_iter(vec![production]);
        table.combine(&incoming, CombinePriority::Append);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(rendered, Ok("production".to_string()));
    }

    #[test]
    fn combine_append_may_not_change_priorities_if_variable_is_disabled() {
        let staging = Field::builder().key("ENVIRONMENT").value("staging").build();
        let production = Field::builder()
            .key("ENVIRONMENT")
            .value("production")
            .active(false)
            .build();
        let table = FieldTable::from_iter(vec![staging]);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(rendered, Ok("staging".to_string()));

        let incoming = FieldTable::from_iter(vec![production]);
        table.combine(&incoming, CombinePriority::Append);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(rendered, Ok("staging".to_string()));
    }

    #[test]
    fn combine_prepend_may_not_change_priorities() {
        let staging = Field::builder().key("ENVIRONMENT").value("staging").build();
        let production = Field::builder()
            .key("ENVIRONMENT")
            .value("production")
            .build();
        let table = FieldTable::from_iter(vec![staging]);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(rendered, Ok("staging".to_string()));

        let incoming = FieldTable::from_iter(vec![production]);
        table.combine(&incoming, CombinePriority::Prepend);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(rendered, Ok("staging".to_string()));
    }

    #[test]
    fn combine_prepend_may_change_priorities_when_disabled() {
        let staging = Field::builder()
            .key("ENVIRONMENT")
            .value("staging")
            .active(false)
            .build();
        let production = Field::builder()
            .key("ENVIRONMENT")
            .value("production")
            .build();
        let table = FieldTable::from_iter(vec![staging]);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(
            rendered,
            Err(srtemplate::Error::VariableNotFound(
                "ENVIRONMENT".to_string()
            ))
        );

        let incoming = FieldTable::from_iter(vec![production]);
        table.combine(&incoming, CombinePriority::Prepend);

        let values = table.template_processor();
        let rendered = values.render("{{ENVIRONMENT}}");
        assert_eq!(rendered, Ok("production".to_string()));
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
    fn test_insert_prepend() {
        let f1 = Field::builder().key("HOST").value("example.com").build();
        let f2 = Field::builder().key("TOKEN").value("123412341234").build();
        let f3 = Field::builder().key("STAGE").value("staging").build();
        let ft = FieldTable::from_iter(vec![f1, f2, f3]);

        let fnew = Field::builder().key("DB").value("staging").build();
        ft.insert_at(&fnew, 0);

        assert_eq!(ft.n_items(), 4);
        assert_eq!("DB", ft.field(0).unwrap().key());
        assert_eq!("HOST", ft.field(1).unwrap().key());
        assert_eq!("TOKEN", ft.field(2).unwrap().key());
        assert_eq!("STAGE", ft.field(3).unwrap().key());
    }

    #[test]
    fn test_insert_prepend_keeps_signal_order() {
        let arc = Arc::new(Mutex::new(Vec::new()));

        let f1 = Field::builder().key("HOST").value("example.com").build();
        let f2 = Field::builder().key("TOKEN").value("123412341234").build();
        let f3 = Field::builder().key("STAGE").value("staging").build();
        let ft = FieldTable::from_iter(vec![f1.clone(), f2.clone(), f3.clone()]);

        let sub_arc = Arc::clone(&arc);
        ft.connect_local("changed", true, move |val| {
            let mut messages = sub_arc.lock().expect("Cannot lock the mutex");
            messages.push(val[1].get::<String>().expect("Value not a string?"));
            None
        });

        // Let's check the current state of the field table.
        f1.set_masked(true);
        f2.set_masked(true);
        f3.set_masked(true);
        {
            let messages = arc.lock().expect("Cannot lock the mutex");
            assert_eq!(*messages, vec!["[0].masked", "[1].masked", "[2].masked"]);
        }

        // Reset to default and clear check array.
        f1.set_masked(false);
        f2.set_masked(false);
        f3.set_masked(false);
        {
            let mut messages = arc.lock().expect("Cannot lock the mutex");
            messages.clear();
        }

        let fnew = Field::builder().key("DB").value("staging").build();
        ft.insert_at(&fnew, 0);

        // The order must have swapped.
        f1.set_masked(true);
        f2.set_masked(true);
        f3.set_masked(true);
        fnew.set_masked(true);
        {
            let messages = arc.lock().expect("Cannot lock the mutex");
            assert_eq!(
                *messages,
                vec![
                    "[0]",
                    "[1].masked",
                    "[2].masked",
                    "[3].masked",
                    "[0].masked"
                ]
            );
        }
    }

    #[test]
    fn test_insert_mid() {
        let f1 = Field::builder().key("HOST").value("example.com").build();
        let f2 = Field::builder().key("TOKEN").value("123412341234").build();
        let f3 = Field::builder().key("STAGE").value("staging").build();
        let ft = FieldTable::from_iter(vec![f1, f2, f3]);

        let fnew = Field::builder().key("DB").value("staging").build();
        ft.insert_at(&fnew, 2);

        assert_eq!(ft.n_items(), 4);
        assert_eq!("HOST", ft.field(0).unwrap().key());
        assert_eq!("TOKEN", ft.field(1).unwrap().key());
        assert_eq!("DB", ft.field(2).unwrap().key());
        assert_eq!("STAGE", ft.field(3).unwrap().key());
    }

    #[test]
    fn test_insert_mid_keeps_signal_order() {
        let arc = Arc::new(Mutex::new(Vec::new()));

        let f1 = Field::builder().key("HOST").value("example.com").build();
        let f2 = Field::builder().key("TOKEN").value("123412341234").build();
        let f3 = Field::builder().key("STAGE").value("staging").build();
        let ft = FieldTable::from_iter(vec![f1.clone(), f2.clone(), f3.clone()]);

        let sub_arc = Arc::clone(&arc);
        ft.connect_local("changed", true, move |val| {
            let mut messages = sub_arc.lock().expect("Cannot lock the mutex");
            messages.push(val[1].get::<String>().expect("Value not a string?"));
            None
        });

        // Let's check the current state of the field table.
        f1.set_masked(true);
        f2.set_masked(true);
        f3.set_masked(true);
        {
            let messages = arc.lock().expect("Cannot lock the mutex");
            assert_eq!(*messages, vec!["[0].masked", "[1].masked", "[2].masked"]);
        }

        // Reset to default and clear check array.
        f1.set_masked(false);
        f2.set_masked(false);
        f3.set_masked(false);
        {
            let mut messages = arc.lock().expect("Cannot lock the mutex");
            messages.clear();
        }

        let fnew = Field::builder().key("DB").value("staging").build();
        ft.insert_at(&fnew, 2);

        // The order must have swapped.
        f1.set_masked(true);
        f2.set_masked(true);
        f3.set_masked(true);
        fnew.set_masked(true);
        {
            let messages = arc.lock().expect("Cannot lock the mutex");
            assert_eq!(
                *messages,
                vec![
                    "[2]",
                    "[0].masked",
                    "[1].masked",
                    "[3].masked",
                    "[2].masked"
                ]
            );
        }
    }

    #[test]
    fn test_insert_append() {
        let f1 = Field::builder().key("HOST").value("example.com").build();
        let f2 = Field::builder().key("TOKEN").value("123412341234").build();
        let f3 = Field::builder().key("STAGE").value("staging").build();
        let ft = FieldTable::from_iter(vec![f1, f2, f3]);

        let fnew = Field::builder().key("DB").value("staging").build();
        ft.insert_at(&fnew, 4);

        assert_eq!(ft.n_items(), 4);
        assert_eq!("HOST", ft.field(0).unwrap().key());
        assert_eq!("TOKEN", ft.field(1).unwrap().key());
        assert_eq!("STAGE", ft.field(2).unwrap().key());
        assert_eq!("DB", ft.field(3).unwrap().key());
    }

    #[test]
    fn test_insert_append_keeps_signal_order() {
        let arc = Arc::new(Mutex::new(Vec::new()));

        let f1 = Field::builder().key("HOST").value("example.com").build();
        let f2 = Field::builder().key("TOKEN").value("123412341234").build();
        let f3 = Field::builder().key("STAGE").value("staging").build();
        let ft = FieldTable::from_iter(vec![f1.clone(), f2.clone(), f3.clone()]);

        let sub_arc = Arc::clone(&arc);
        ft.connect_local("changed", true, move |val| {
            let mut messages = sub_arc.lock().expect("Cannot lock the mutex");
            messages.push(val[1].get::<String>().expect("Value not a string?"));
            None
        });

        // Let's check the current state of the field table.
        f1.set_masked(true);
        f2.set_masked(true);
        f3.set_masked(true);
        {
            let messages = arc.lock().expect("Cannot lock the mutex");
            assert_eq!(*messages, vec!["[0].masked", "[1].masked", "[2].masked"]);
        }

        // Reset to default and clear check array.
        f1.set_masked(false);
        f2.set_masked(false);
        f3.set_masked(false);
        {
            let mut messages = arc.lock().expect("Cannot lock the mutex");
            messages.clear();
        }

        let fnew = Field::builder().key("DB").value("staging").build();
        ft.insert_at(&fnew, 4);

        // The order must have swapped.
        f1.set_masked(true);
        f2.set_masked(true);
        f3.set_masked(true);
        fnew.set_masked(true);
        {
            let messages = arc.lock().expect("Cannot lock the mutex");
            assert_eq!(
                *messages,
                vec![
                    "[3]",
                    "[0].masked",
                    "[1].masked",
                    "[2].masked",
                    "[3].masked"
                ]
            );
        }
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
            assert_eq!(pos, 0);
            assert_eq!(removed, 0);
            assert_eq!(added, 1);
        }

        {
            table.insert(&field2);
            assert_eq!(table.n_items(), 2);
            let (pos, removed, added) = inserts.lock().unwrap().last().unwrap().to_owned();
            assert_eq!(pos, 1);
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

        assert_emits_signal(&table1, "changed", || {
            table1.field(0).unwrap().set_value("application/json")
        });
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
        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("API_ROOT")
                .value("http://localhost:8000")
                .active(false)
                .build(),
        ]);

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

        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("Location")
                .value("{{ API_ROOT }}/v1/users")
                .build(),
        ]);
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
        assert_eq!(table.n_items(), 1);
        assert_field(&table.field(0).unwrap(), "cat_id", "15", true, false);
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

    #[test]
    fn test_dup() {
        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("user-agent")
                .value("mozilla/5.0")
                .build(),
            Field::builder().key("accept").value("text/html").build(),
        ]);
        let duped = table.dup();
        assert_eq!(table.n_items(), duped.n_items());
        assert_eq!(table.field(0).unwrap().key(), duped.field(0).unwrap().key());
        assert_eq!(
            table.field(0).unwrap().value(),
            duped.field(0).unwrap().value()
        );
        assert_eq!(table.field(1).unwrap().key(), duped.field(1).unwrap().key());
        assert_eq!(
            table.field(1).unwrap().value(),
            duped.field(1).unwrap().value()
        );
    }

    #[test]
    pub fn test_dup_emits_separate_change_signals() {
        let table = FieldTable::from_iter(vec![
            Field::builder()
                .key("user-agent")
                .value("mozilla/5.0")
                .build(),
            Field::builder().key("accept").value("text/html").build(),
        ]);
        let duped = table.dup();
        assert_emits_signal(&table, "changed", || {
            table.field(0).unwrap().set_key("Accept")
        });
        assert_emits_signal(&duped, "changed", || {
            duped.field(0).unwrap().set_key("Accept")
        });
        assert_not_emits_signal(&table, "changed", || {
            duped.field(0).unwrap().set_value("text/html")
        });
        assert_not_emits_signal(&duped, "changed", || {
            table.field(0).unwrap().set_value("text/html")
        });
    }

    fn assert_field(f: &Field, key: &str, value: &str, active: bool, masked: bool) {
        assert_eq!(f.key(), key);
        assert_eq!(f.value(), value);
        assert_eq!(f.active(), active);
        assert_eq!(f.masked(), masked);
    }
}
