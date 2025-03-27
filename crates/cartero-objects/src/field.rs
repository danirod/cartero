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

use glib::prelude::*;
use glib::subclass::prelude::*;

glib::wrapper! {
    pub struct Field(ObjectSubclass<imp::Field>);
}

mod imp {
    use std::cell::RefCell;

    use super::*;
    use glib::Properties;

    #[derive(Properties)]
    #[properties(wrapper_type = super::Field)]
    pub struct Field {
        #[property(get, set)]
        key: RefCell<String>,

        #[property(get, set)]
        value: RefCell<String>,

        #[property(get, set, default = true)]
        active: RefCell<bool>,

        #[property(get, set, default = false)]
        masked: RefCell<bool>,
    }

    impl Default for Field {
        fn default() -> Self {
            Self {
                key: Default::default(),
                value: Default::default(),
                active: true.into(),
                masked: false.into(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Field {
        const NAME: &'static str = "CarteroField";
        type Type = super::Field;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Field {}
}

#[cfg(test)]
mod tests {

    use glib::Object;

    use crate::utils::test::assert_emits_signal;

    use super::*;

    #[test]
    pub fn test_valid_builder() {
        let field: Field = Object::builder()
            .property("key", "User-Agent")
            .property("value", "Mozilla/5.0")
            .build();
        assert_eq!(field.key(), "User-Agent");
        assert_eq!(field.value(), "Mozilla/5.0");
        assert!(field.active());
        assert!(!field.masked());
    }

    #[test]
    pub fn test_notifies_changes() {
        let field: Field = Object::builder()
            .property("key", "User-Agent")
            .property("value", "Mozilla/5.0")
            .build();

        assert_emits_signal(&field, "notify", || field.set_key("Accept"));
        assert_emits_signal(&field, "notify", || field.set_value("text/html"));
        assert_emits_signal(&field, "notify", || field.set_active(false));
        assert_emits_signal(&field, "notify", || field.set_masked(true));
    }
}
