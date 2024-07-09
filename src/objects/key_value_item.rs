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

use glib::{object::ObjectExt, Object};

use crate::client::KeyValueData;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use glib::subclass::Signal;
    use glib::Properties;
    use gtk::glib;
    use gtk::glib::prelude::*;
    use gtk::glib::subclass::prelude::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type = super::KeyValueItem)]
    pub struct KeyValueItem {
        #[property(get, set)]
        active: RefCell<bool>,
        #[property(get, set)]
        secret: RefCell<bool>,
        #[property(get, set)]
        ignored: RefCell<bool>,

        #[property(get, set)]
        header_name: RefCell<String>,
        #[property(get, set)]
        header_value: RefCell<String>,
        #[property(get, set)]
        dirty: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeyValueItem {
        const NAME: &'static str = "CarteroKeyValueItem";
        type Type = super::KeyValueItem;
    }

    #[glib::derived_properties]
    impl ObjectImpl for KeyValueItem {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_signals();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("changed").build()])
        }
    }
}

glib::wrapper! {
    pub struct KeyValueItem(ObjectSubclass<imp::KeyValueItem>);
}

impl KeyValueItem {
    pub(self) fn setup_signals(&self) {
        self.connect_header_name_notify(|item| {
            if !item.dirty() {
                item.set_dirty(true);
                item.set_active(true);
            }
            item.emit_by_name::<()>("changed", &[]);
        });
        self.connect_header_value_notify(|item| {
            if !item.dirty() {
                item.set_dirty(true);
                item.set_active(true);
            }
            item.emit_by_name::<()>("changed", &[]);
        });
        self.connect_active_notify(|item| {
            item.emit_by_name::<()>("changed", &[]);
        });
        self.connect_secret_notify(|item| {
            item.emit_by_name::<()>("changed", &[]);
        });
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_data(name: &str, value: &KeyValueData) -> Self {
        let header = Self::new();
        header.set_header_name(name);
        header.set_header_value(value.value.clone());
        header.set_active(value.active);
        header.set_secret(value.secret);
        header
    }

    pub fn new_with_value(name: &str, value: &str) -> Self {
        let header = Self::new();
        header.set_header_name(name);
        header.set_header_value(value);
        header.set_active(true);
        header
    }

    // For a header to be actually usable, it must be checked, and also it must have a header name
    // properly set. We could argue that having an empty value is also dumb, but the spec
    // technically allows this.
    pub fn is_usable(&self) -> bool {
        self.active() && !self.header_name().is_empty()
    }
}

impl Default for KeyValueItem {
    fn default() -> Self {
        Object::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use super::KeyValueItem;

    #[test]
    pub fn test_header_properties() {
        let header = KeyValueItem::default();
        assert_eq!(header.header_name(), "");
        assert_eq!(header.header_value(), "");
        assert!(!header.active());

        header.set_header_name("Content-Type");
        header.set_header_value("text/plain");
        header.set_active(true);
        assert_eq!(header.header_name(), "Content-Type");
        assert_eq!(header.header_value(), "text/plain");
        assert!(header.active());

        header.set_header_name("Accept");
        assert_eq!(header.header_name(), "Accept");
        assert_eq!(header.header_value(), "text/plain");
        assert!(header.active());
    }
}
