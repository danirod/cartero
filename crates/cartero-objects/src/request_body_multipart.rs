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

#[cfg(test)]
mod tests {
    use gio::prelude::ListModelExt;
    use glib::object::CastNone;

    use crate::{Field, RequestBodyDataExt, RequestBodyType};

    use super::RequestBodyMultipart;

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
