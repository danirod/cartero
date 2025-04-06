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
    /// Body payload with key-value pairs encoded as an URL string.
    ///
    /// The preferred content-type is `application/x-www-form-urlencoded`, and
    /// it is usually the value of the `Content-Type` header. The payload
    /// consists of a set of key-value pairs (internally encoded using
    /// [`FieldTable`s][super::FieldTable]), encoded as a string as defined
    /// by the [chapter 5 of the URL Standard][url-standard-ch05].
    ///
    /// [url-standard-ch05]: https://url.spec.whatwg.org/#application/x-www-form-urlencoded
    ///
    /// ## Properties
    ///
    /// `params`: the field table in use.
    ///
    /// ## Setting up an instance
    ///
    /// - Use the `default` or the `new` method to create empty urlencoded payloads.
    /// - Use the [`from_table`][RequestBodyUrlencoded::from_table] function to
    ///   initialise the payload to a given table.
    pub struct RequestBodyUrlencoded(ObjectSubclass<imp::RequestBodyUrlencoded>) @extends crate::RequestBodyData;
}

impl Default for RequestBodyUrlencoded {
    fn default() -> Self {
        Object::new()
    }
}

impl RequestBodyUrlencoded {
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
    #[properties(wrapper_type = super::RequestBodyUrlencoded)]
    pub struct RequestBodyUrlencoded {
        #[property(get, set)]
        params: RefCell<FieldTable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyUrlencoded {
        const NAME: &'static str = "CarteroRequestBodyUrlencoded";
        type Type = super::RequestBodyUrlencoded;
        type ParentType = crate::RequestBodyData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBodyUrlencoded {}

    impl RequestBodyDataImpl for RequestBodyUrlencoded {
        fn body_type(&self) -> crate::RequestBodyType {
            crate::RequestBodyType::UrlEncoded
        }
    }
}

#[cfg(test)]
mod tests {
    use gio::prelude::ListModelExt;
    use glib::object::CastNone;

    use crate::{Field, RequestBodyDataExt, RequestBodyType};

    use super::RequestBodyUrlencoded;

    #[test]
    pub fn test_default() {
        let body = RequestBodyUrlencoded::default();
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
        let body: RequestBodyUrlencoded = RequestBodyUrlencoded::default();
        assert_eq!(body.body_type(), RequestBodyType::UrlEncoded);
    }
}
