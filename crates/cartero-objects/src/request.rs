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
