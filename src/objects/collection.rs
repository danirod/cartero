// Copyright 2024 the Cartero authors
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

use glib::{object::Cast, types::StaticType, Object};
use gtk::{gio::ListStore, prelude::ListModelExt};

use super::KeyValueItem;

mod imp {
    use std::cell::{OnceCell, RefCell};

    use glib::Properties;
    use gtk::gio::ListStore;
    use gtk::glib::prelude::*;
    use gtk::glib::subclass::prelude::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type = super::Collection)]
    pub struct Collection {
        #[property(get, set, default = "")]
        title: RefCell<String>,

        #[property(get, set, default = "")]
        description: RefCell<String>,

        #[property(get, set, default = "")]
        version: RefCell<String>,

        #[property(get, set)]
        pub(super) variables: OnceCell<ListStore>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Collection {
        const NAME: &'static str = "CarteroCollection";
        type Type = super::Collection;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Collection {}
}

glib::wrapper! {
    pub struct Collection(ObjectSubclass<imp::Collection>);
}

impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}

impl Collection {
    pub fn new() -> Self {
        let empty_collection = ListStore::with_type(KeyValueItem::static_type());
        Object::builder()
            .property("variables", empty_collection)
            .build()
    }

    pub fn add_variable(&self, var: &KeyValueItem) {
        self.variables().append(var);
    }

    pub fn variable_count(&self) -> u32 {
        self.variables().n_items()
    }

    pub fn variable_get(&self, pos: u32) -> Option<KeyValueItem> {
        self.variables()
            .item(pos)
            .and_then(|obj| obj.downcast::<KeyValueItem>().ok())
    }

    pub fn variable_del(&self, pos: u32) -> Option<KeyValueItem> {
        if let Some(obj) = self.variable_get(pos) {
            self.variables().remove(pos);
            Some(obj)
        } else {
            None
        }
    }

    pub fn variables_list(&self) -> Vec<KeyValueItem> {
        (0..self.variable_count())
            .map(|i| self.variable_get(i))
            .map(Option::unwrap)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::objects::KeyValueItem;

    use super::Collection;

    #[test]
    pub fn test_collections_can_have_variables() {
        let collection = Collection::new();

        let variable = {
            let v = KeyValueItem::default();
            v.set_header_name("token");
            v.set_header_value("12341234");
            v.set_active(true);
            v.set_secret(true);
            v
        };

        assert_eq!(0, collection.variable_count());
        assert!(collection.variable_get(0).is_none());

        collection.add_variable(&variable);
        assert_eq!(1, collection.variable_count());

        let variable = collection.variable_get(0).unwrap();
        assert_eq!("token", variable.header_name());
        assert_eq!("12341234", variable.header_value());
        assert!(variable.active());
        assert!(variable.secret());

        let var = collection.variable_del(0);
        assert!(var.is_some());
        assert_eq!(collection.variable_count(), 0);
        assert!(collection.variable_get(0).is_none());
    }
}
