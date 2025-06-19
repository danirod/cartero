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

use glib::subclass::prelude::*;
use glib::{prelude::*, Object};

glib::wrapper! {
    /// A key-valued string container with additional metadata.
    ///
    /// Fields are used to represent headers, parameters, payloads and
    /// variables within the application. They are usually grouped in
    /// [tables][crate::FieldTable], allowing multiple fields to be combined
    /// to detect things such as duplicates or parameter arrays.
    ///
    /// ## Properties
    ///
    /// - `key`: the name of the field.
    /// - `value`: the value of the field.
    /// - `active`: whether the field is meant to be used (for instance,
    ///   during a request).
    /// - `masked`: whether the value of the field is meant to be treated as
    ///   a password. If this value is true, entries and other text areas
    ///   designed to interact with the value of a field should treat the
    ///   value as a password.
    ///
    /// ## Building a Field
    ///
    /// To create an empty Field, the `Default` trait can be used. The
    /// properties can change value later.
    ///
    /// ```
    /// use cartero_objects::Field;
    ///
    /// let field = Field::default();
    /// field.set_key("User-Agent");
    /// field.set_value("Mozilla/5.0");
    /// ```
    ///
    /// However, to programatically create Fields, there is a shortcut.
    /// A pair of `AsRef<str>` can be given to the `.from()` function in order
    /// to quickly initialise the key and value to some known strings. The
    /// default active and masked value are kept:
    ///
    /// ```
    /// use cartero_objects::Field;
    ///
    /// let field = Field::from(("User-Agent", "Mozilla/5.0"));
    /// assert_eq!(field.key(), "User-Agent");
    /// assert_eq!(field.value(), "Mozilla/5.0");
    /// ```
    pub struct Field(ObjectSubclass<imp::Field>);
}

impl Field {
    pub fn builder<K, V>(key: K, value: V) -> builder::FieldBuilder
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        builder::FieldBuilder::new(key, value)
    }
}

impl<T> From<(T, T)> for Field
where
    T: AsRef<str>,
{
    fn from(value: (T, T)) -> Self {
        Object::builder()
            .property("key", value.0.as_ref())
            .property("value", value.1.as_ref())
            .build()
    }
}

impl Default for Field {
    fn default() -> Self {
        Object::new()
    }
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

mod builder {
    use super::*;
    use glib::object::ObjectBuilder;

    pub struct FieldBuilder {
        builder: ObjectBuilder<'static, Field>,
    }

    impl FieldBuilder {
        pub fn new<K, V>(key: K, value: V) -> Self
        where
            K: AsRef<str>,
            V: AsRef<str>,
        {
            let builder = glib::Object::builder()
                .property("key", key.as_ref())
                .property("value", value.as_ref());
            Self { builder }
        }

        pub fn build(self) -> Field {
            self.builder.build()
        }

        pub fn active(mut self, active: bool) -> Self {
            self.builder = self.builder.property("active", active);
            self
        }

        pub fn masked(mut self, masked: bool) -> Self {
            self.builder = self.builder.property("masked", masked);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test::assert_emits_signal;

    use super::*;

    #[test]
    pub fn test_valid_defaults() {
        let field = Field::builder("User-Agent", "Mozilla/5.0").build();
        assert_eq!(field.key(), "User-Agent");
        assert_eq!(field.value(), "Mozilla/5.0");
        assert!(field.active());
        assert!(!field.masked());
    }

    #[test]
    pub fn test_valid_builder() {
        let field = Field::builder("User-Agent", "Mozilla/5.0")
            .active(false)
            .masked(true)
            .build();
        assert_eq!(field.key(), "User-Agent");
        assert_eq!(field.value(), "Mozilla/5.0");
        assert!(!field.active());
        assert!(field.masked());
    }

    #[test]
    pub fn test_notifies_changes() {
        let field = Field::builder("User-Agent", "Mozilla/5.0").build();
        assert_emits_signal(&field, "notify", || field.set_key("Accept"));
        assert_emits_signal(&field, "notify", || field.set_value("text/html"));
        assert_emits_signal(&field, "notify", || field.set_active(false));
        assert_emits_signal(&field, "notify", || field.set_masked(true));
    }
}
