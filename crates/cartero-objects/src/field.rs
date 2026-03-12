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

use glib::subclass::prelude::*;
use glib::{Object, prelude::*};

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
    pub fn builder() -> builder::FieldBuilder {
        builder::FieldBuilder::default()
    }

    pub fn dup(&self) -> Self {
        builder::FieldBuilder::default()
            .key(self.key())
            .value(self.value())
            .active(self.active())
            .masked(self.masked())
            .build()
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
    use std::{cell::RefCell, sync::OnceLock};

    use super::*;
    use glib::{Properties, subclass::Signal};

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
    impl ObjectImpl for Field {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_key_notify(|field| {
                field.emit_by_name::<()>("changed", &[&"key"]);
            });
            self.obj().connect_value_notify(|field| {
                field.emit_by_name::<()>("changed", &[&"value"]);
            });
            self.obj().connect_active_notify(|field| {
                field.emit_by_name::<()>("changed", &[&"active"]);
            });
            self.obj().connect_masked_notify(|field| {
                field.emit_by_name::<()>("changed", &[&"masked"]);
            });
        }

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
}

mod builder {
    use super::*;
    use glib::object::ObjectBuilder;

    pub struct FieldBuilder {
        builder: ObjectBuilder<'static, Field>,
    }

    impl Default for FieldBuilder {
        fn default() -> Self {
            Self {
                builder: Object::builder(),
            }
        }
    }

    impl FieldBuilder {
        pub fn build(self) -> Field {
            self.builder.build()
        }

        pub fn key<T>(mut self, key: T) -> Self
        where
            T: AsRef<str>,
        {
            self.builder = self.builder.property("key", key.as_ref());
            self
        }

        pub fn value<T>(mut self, value: T) -> Self
        where
            T: AsRef<str>,
        {
            self.builder = self.builder.property("value", value.as_ref());
            self
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
    use crate::utils::test::{assert_emits_signal, assert_not_emits_signal};

    use super::*;

    #[test]
    pub fn test_valid_defaults() {
        let field = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        assert_eq!(field.key(), "User-Agent");
        assert_eq!(field.value(), "Mozilla/5.0");
        assert!(field.active());
        assert!(!field.masked());
    }

    #[test]
    pub fn test_valid_builder() {
        let field = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
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
        let field = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        assert_emits_signal(&field, "notify", || field.set_key("Accept"));
        assert_emits_signal(&field, "notify", || field.set_value("text/html"));
        assert_emits_signal(&field, "notify", || field.set_active(false));
        assert_emits_signal(&field, "notify", || field.set_masked(true));
    }

    #[test]
    pub fn test_emits_changes_on_change() {
        let field = Field::builder().build();
        assert_emits_signal(&field, "changed", || field.set_key("Accept"));
        assert_emits_signal(&field, "changed", || field.set_value("text/html"));
        assert_emits_signal(&field, "changed", || field.set_active(false));
        assert_emits_signal(&field, "changed", || field.set_masked(true));
    }

    #[test]
    pub fn test_dup() {
        let field = Field::builder()
            .key("user-agent")
            .value("mozilla/5.0")
            .active(false)
            .masked(true)
            .build();
        let duped = field.dup();
        assert_eq!(field.key(), "user-agent");
        assert_eq!(field.value(), "mozilla/5.0");
        assert!(!field.active());
        assert!(field.masked());

        assert_eq!(duped.key(), "user-agent");
        assert_eq!(duped.value(), "mozilla/5.0");
        assert!(!duped.active());
        assert!(duped.masked());
    }

    #[test]
    pub fn test_dup_emits_separate_change_signals() {
        let field = Field::builder()
            .key("user-agent")
            .value("mozilla/5.0")
            .build();
        let duped = field.dup();
        assert_emits_signal(&field, "changed", || field.set_key("Accept"));
        assert_emits_signal(&duped, "changed", || duped.set_key("Accept"));

        assert_not_emits_signal(&field, "changed", || duped.set_value("text/html"));
        assert_not_emits_signal(&duped, "changed", || field.set_value("text/html"));
    }
}
