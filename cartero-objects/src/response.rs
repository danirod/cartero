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
    /// The high order class that represents a response.
    ///
    ///  A `Response` class is made of different information components that
    /// are received by the HTTP client and that are used in order to present
    /// the response to the user, for instance, via the user interface.
    /// The response is not usually modifiable.
    ///
    /// ## Properties
    ///
    /// - `body`: the body of the response (may be empty, for instance, during
    ///   an HTTP HEAD request or if the server returns 204).
    /// - `duration`: the length in milliseconds the request took to complete.
    /// - `headers`: a FieldTable with the response headers sent by the server.
    /// - `size`: the amount in bytes of data contained in the body.
    /// - `status-code`: the numerical status code returned by the server.
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
