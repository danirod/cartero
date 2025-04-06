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
    /// The high order class that represents a request.
    ///
    /// A `Request` class is made of the different information components that
    /// are needed in order to fully craft an HTTP request. These elements are
    /// meant to be presented to the user via the user interface to let the
    /// user get or change the values.
    ///
    /// ## Properties
    ///
    /// - `authentication`: a [RequestAuthentication][super::RequestAuthentication]
    ///   object to interact with the authentication data. This is later treated
    ///   as the `Authorization` header when sending a request.
    /// - `body`: a [RequestBody][super::RequestBody] object to interact with
    ///   the payload that some HTTP requests can carry when being performed.
    /// - `headers`: a [FieldTable][super::FieldTable] to collect the headers
    ///   to be added to a request.
    /// - `method`: a [RequestMethod][super::RequestMethod] enum value used
    ///   to indicate the verb.
    /// - `params`: a [FieldTable][super::FieldTable] that collects additional
    ///   query parameters. These are added to the URL during a request as
    ///   long as the field is enabled.
    /// - `url`: a String with the target URL where the request is pointing to.
    /// - `variables`: a [FieldTable][super::FieldTable] with variables that
    ///   are interpolated before sending an HTTP request, in order to un-hardcode
    ///   common things such as API tokens, passwords, roots...
    ///
    /// ## URL vs Params
    ///
    /// Both fields contradict themselves. The URL is a String that may carry
    /// a query string (the `?` character followed by zero, one or more
    /// urlencoded key-value pairs). The params table may also carry extra
    /// fields.
    ///
    /// It's not up to this crate to decide which one to pick. The values may
    /// be concatted, the table may carry only disabled parameters, or the URL
    /// may be stripped of the querystring and every parameter may be added
    /// into the table. But this is a task for caller code (such as the user
    /// interface or the file serialization API).
    pub struct Request(ObjectSubclass<imp::Request>);
}

impl Default for Request {
    fn default() -> Self {
        Object::builder().build()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use crate::{field_table::FieldTable, RequestAuthentication, RequestBody, RequestMethod};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Request)]
    pub struct Request {
        #[property(get, set, builder(RequestMethod::default()))]
        method: RefCell<RequestMethod>,

        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set)]
        params: RefCell<FieldTable>,

        #[property(get, set)]
        headers: RefCell<FieldTable>,

        #[property(get, set)]
        variables: RefCell<FieldTable>,

        #[property(get)]
        authentication: RefCell<RequestAuthentication>,

        #[property(get)]
        body: RefCell<RequestBody>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Request {
        const NAME: &'static str = "CarteroRequest";
        type Type = super::Request;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Request {}
}
