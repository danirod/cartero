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
    pub struct Response(ObjectSubclass<imp::Response>);
}

mod imp {
    use super::*;
    use std::cell::RefCell;

    use glib::Properties;

    use crate::{FieldTable, Request};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Response)]
    pub struct Response {
        #[property(get, set)]
        request: RefCell<Request>,
        #[property(get, set)]
        status_code: RefCell<u32>,
        #[property(get, set)]
        duration: RefCell<u64>,
        #[property(get, set)]
        size: RefCell<u64>,
        #[property(get, set)]
        headers: RefCell<FieldTable>,
        #[property(get, set, nullable)]
        body: RefCell<Option<glib::Bytes>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Response {
        const NAME: &'static str = "CarteroResponse";
        type Type = super::Response;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Response {}
}
