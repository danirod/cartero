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

use crate::{FieldTable, Request};

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

impl Response {
    pub fn builder(request: &Request) -> builder::ResponseBuilder {
        builder::ResponseBuilder::new(request)
    }
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

mod builder {
    use glib::{object::ObjectBuilder, Object};

    use super::*;

    pub struct ResponseBuilder {
        builder: ObjectBuilder<'static, Response>,
    }

    impl ResponseBuilder {
        pub fn new(request: &Request) -> Self {
            Self {
                builder: Object::builder().property("request", request),
            }
        }

        pub fn build(self) -> Response {
            self.builder.build()
        }

        pub fn status_code(mut self, code: u32) -> Self {
            self.builder = self.builder.property("status-code", code);
            self
        }

        pub fn duration(mut self, duration: u64) -> Self {
            self.builder = self.builder.property("duration", duration);
            self
        }

        pub fn size(mut self, size: u64) -> Self {
            self.builder = self.builder.property("size", size);
            self
        }

        pub fn headers(mut self, table: &FieldTable) -> Self {
            self.builder = self.builder.property("headers", table);
            self
        }

        pub fn body(mut self, body: &[u8]) -> Self {
            let bytes = glib::Bytes::from(body);
            self.builder = self.builder.property("body", bytes);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use gio::prelude::ListModelExt;

    use crate::{Field, RequestMethod};

    use super::*;

    fn request() -> Request {
        Request::builder("https://www.example.com/api/users", RequestMethod::Get).build()
    }

    #[test]
    pub fn test_builder() {
        let response = Response::builder(&request()).build();
        assert_eq!(response.status_code(), 0);
        assert_eq!(response.duration(), 0);
        assert_eq!(response.size(), 0);
        assert_eq!(response.headers().n_items(), 0);
        assert!(response.body().is_none());
    }

    #[test]
    pub fn test_builder_full() {
        let response_headers = FieldTable::from_iter(vec![
            Field::builder("Server", "nginx/1.0").build(),
            Field::builder("Content-Type", "text/plain").build(),
        ]);
        let response = Response::builder(&request())
            .status_code(404)
            .duration(532)
            .size(1234)
            .headers(&response_headers)
            .body(b"Not found!")
            .build();
        assert_eq!(response.status_code(), 404);
        assert_eq!(response.duration(), 532);
        assert_eq!(response.size(), 1234);
        assert_eq!(response.headers().n_items(), 2);
        let data = response.body().unwrap().into_data();
        assert_eq!(data.as_ref(), b"Not found!");
    }
}
