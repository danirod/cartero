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

use crate::FieldTable;

glib::wrapper! {
    /// Body payload with a rich multipart stream encoded.
    ///
    /// The preferred content-type is `multipart/form-data`. This is a more
    /// complex content-type that is defined by the RFC 7578 spec. It is a
    /// complex key-value form, where every value can have its own content
    /// type and additional metadata, supporting both inline strings, but also
    /// raw files.
    ///
    /// (Note that attaching files to a `RequestBodyMultipart` is still not
    /// supported, but it is a pending feature).
    ///
    /// ## Properties
    ///
    /// - `params`: the field table in use.
    ///
    /// Currently, the `params` property is a table of `Field` instances. Once
    /// support for file attachments is added, this may change to allow for
    /// richer types.
    ///
    /// ## Setting up an instance
    ///
    /// - Use the `default` or the `new` method to create empty payloads.
    /// - Use the [`from_table`][RequestBodyMultipart::from_table] function to
    ///   initialise the payload to the given table.
    pub struct RequestBodyMultipart(ObjectSubclass<imp::RequestBodyMultipart>) @extends crate::RequestBodyData;
}

impl Default for RequestBodyMultipart {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl RequestBodyMultipart {
    /// Create a new payload with an empty field table with no data.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> builder::RequestBodyMultipartBuilder {
        builder::RequestBodyMultipartBuilder::default()
    }

    /// Create a new payload with the given table as initial data.
    pub fn from_table(table: &FieldTable) -> Self {
        Object::builder().property("params", table).build()
    }
}

mod imp {
    use glib::Properties;

    use super::*;

    use std::cell::RefCell;

    use crate::{FieldTable, RequestBodyDataImpl};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestBodyMultipart)]
    pub struct RequestBodyMultipart {
        #[property(get, set)]
        params: RefCell<FieldTable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyMultipart {
        const NAME: &'static str = "CarteroRequestBodyMultipart";
        type Type = super::RequestBodyMultipart;
        type ParentType = crate::RequestBodyData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBodyMultipart {}

    impl RequestBodyDataImpl for RequestBodyMultipart {
        fn body_type(&self) -> crate::RequestBodyType {
            crate::RequestBodyType::Multipart
        }
    }
}

mod builder {
    use glib::object::ObjectBuilder;

    use crate::Field;

    use super::*;

    pub struct RequestBodyMultipartBuilder {
        builder: ObjectBuilder<'static, RequestBodyMultipart>,
        field_table: FieldTable,
    }

    impl Default for RequestBodyMultipartBuilder {
        fn default() -> Self {
            let builder = Object::builder();
            Self {
                builder,
                field_table: FieldTable::default(),
            }
        }
    }

    impl RequestBodyMultipartBuilder {
        pub fn build(self) -> RequestBodyMultipart {
            let object = self.builder.build();
            object.set_params(&self.field_table);
            object
        }

        pub fn field(self, field: &Field) -> Self {
            self.field_table.insert(field);
            self
        }

        pub fn params(self, params: &FieldTable) -> Self {
            self.field_table.replace(params);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use gio::prelude::ListModelExt;
    use glib::object::CastNone;

    use crate::{Field, FieldTable, RequestBodyDataExt, RequestBodyType};

    use super::RequestBodyMultipart;

    // TODO: This test sometimes lock the test suite.
    #[test]
    pub fn test_builder_empty() {
        let body = RequestBodyMultipart::builder().build();
        assert_eq!(0, body.params().n_items());
    }

    // TODO: This test sometimes lock the test suite.
    #[test]
    pub fn test_builder_from_fields_table() {
        let field1 = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        let field2 = Field::builder()
            .key("Accept")
            .value("application/json")
            .build();
        let field_table = FieldTable::from_iter(vec![field1, field2]);
        let body = RequestBodyMultipart::builder().params(&field_table).build();
        assert_eq!(2, body.params().n_items());
        assert_eq!("User-Agent", body.params().field(0).unwrap().key());
        assert_eq!("Accept", body.params().field(1).unwrap().key());
    }

    // TODO: This test sometimes lock the test suite.
    #[test]
    pub fn test_builder_adding_fields() {
        let field1 = Field::builder()
            .key("User-Agent")
            .value("Mozilla/5.0")
            .build();
        let field2 = Field::builder()
            .key("Accept")
            .value("application/json")
            .build();
        let body = RequestBodyMultipart::builder()
            .field(&field1)
            .field(&field2)
            .build();
        assert_eq!(2, body.params().n_items());
        assert_eq!("User-Agent", body.params().field(0).unwrap().key());
        assert_eq!("Accept", body.params().field(1).unwrap().key());
    }

    #[test]
    pub fn test_default() {
        let body = RequestBodyMultipart::default();
        assert_eq!(0, body.params().n_items());
        let field = Field::from(("user_id", "1000"));
        body.params().insert(&field);
        assert_eq!(1, body.params().n_items());
        assert_eq!(
            "user_id",
            body.params().item(0).and_downcast::<Field>().unwrap().key()
        );
    }

    #[test]
    pub fn test_body_type() {
        let body = RequestBodyMultipart::default();
        assert_eq!(body.body_type(), RequestBodyType::Multipart);
    }
}
