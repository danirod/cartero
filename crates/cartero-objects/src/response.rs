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

use std::str::FromStr;

use content_disposition::parse_content_disposition;
use glib::prelude::*;
use glib::subclass::prelude::*;
use http::Uri;

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

    fn test_content_type(&self, query: &str) -> bool {
        match self.headers().find_by_name_icase("content-type") {
            Some(headers) => match &headers[..] {
                [field] => field.contains(query),
                _ => false,
            },
            None => false,
        }
    }

    pub fn file_name(&self) -> String {
        let disposition = self
            .headers()
            .find_by_name_icase("content-disposition")
            .and_then(|headers| {
                if headers.len() == 1 {
                    Some(headers[0].clone())
                } else {
                    None
                }
            })
            .map(|header| parse_content_disposition(header.as_str()))
            .and_then(|disposition| disposition.filename_full());

        disposition.unwrap_or_else(|| {
            let full_url = self.request().url();
            let url = full_url
                .split_once('?')
                .map(|(start, _)| start)
                .unwrap_or(&full_url);
            match Uri::from_str(url) {
                Ok(uri) => uri
                    .path()
                    .rsplit_once("/")
                    .map(|(_, end)| end)
                    .unwrap_or(url)
                    .to_owned(),
                Err(_) => url.to_owned(),
            }
        })
    }

    pub fn is_json(&self) -> bool {
        self.test_content_type("/json") || self.test_content_type("+json")
    }

    pub fn is_xml(&self) -> bool {
        self.test_content_type("/xml") || self.test_content_type("+xml")
    }

    pub fn is_binary(&self) -> bool {
        // TODO: Test
        self.test_content_type("application/octet-stream")
            || self.test_content_type("image/")
            || self.test_content_type("audio/")
            || self.test_content_type("video/")
            || self.test_content_type("haptics/")
            || self.test_content_type("font/")
            || self.body().is_some_and(|body| {
                // Check for non-printable characters, but respect characters in range
                // 0x08 to 0x0D: these are \n, \r, \t, and technically these are
                // printable.
                body.clone()
                    .to_vec()
                    .into_iter()
                    .find(|ch| *ch < 0x08 || (*ch >= 0x0D && *ch < 0x20))
                    .is_some()
            })
    }

    /// Returns a safe representation of the body, in a way that can be presented
    /// by the GtkSourceView that renders bodies. Converts the \0 character with
    /// an <?> because otherwise you would get a GStrInteriorNulError.
    ///
    /// This method is born deprecated. It will not be present in 25.0 because
    /// the response panel will simply refuse to render binary responses that
    /// contain the \0 character and instead will just offer to export the
    /// response.
    ///
    /// It is present because such functionality has not been added yet and we
    /// still need to let things work as they are until a sane exporter is
    /// added.
    #[deprecated = "Don't use it for new code, will be removed in 25.0"]
    pub fn safe_string(&self) -> String {
        let body = self
            .body()
            .map(|body| {
                let vector = body.to_vec();
                String::from_utf8_lossy(&vector).into_owned()
            })
            .unwrap_or_default();
        body.replace("\x00", "�")
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
        effective_url: RefCell<String>,
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

        pub fn effective_url(mut self, url: String) -> Self {
            self.builder = self.builder.property("effective-url", url);
            self
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
    fn file_name_reads_from_url() {
        let response = Response::builder(&request()).build();
        let file_name = response.file_name();
        assert_eq!(file_name, "users");
    }

    #[test]
    fn file_name_strips_query_params() {
        let request = Request::builder(
            "https://www.example.com/index.html?page=12341234",
            RequestMethod::Get,
        )
        .build();
        let response = Response::builder(&request).build();
        let file_name = response.file_name();
        assert_eq!(file_name, "index.html");
    }

    #[test]
    fn file_name_includes_extension() {
        let request =
            Request::builder("https://www.example.com/attachment.jpg", RequestMethod::Get).build();
        let response = Response::builder(&request).build();
        let file_name = response.file_name();
        assert_eq!(file_name, "attachment.jpg");
    }

    #[test]
    fn file_name_deprioritizes_inline_content_disposition() {
        let disposition = Field::builder()
            .key("Content-Disposition")
            .value("inline")
            .build();
        let headers = FieldTable::from_iter([disposition]);
        let response = Response::builder(&request()).headers(&headers).build();
        let file_name = response.file_name();
        assert_eq!(file_name, "users");
    }

    #[test]
    fn file_name_prioritizes_attachment_content_disposition() {
        let disposition = Field::builder()
            .key("Content-Disposition")
            .value("attachment; filename=\"report.json\"")
            .build();
        let headers = FieldTable::from_iter([disposition]);
        let response = Response::builder(&request()).headers(&headers).build();
        let file_name = response.file_name();
        assert_eq!(file_name, "report.json");
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
            Field::builder().key("Server").value("nginx/1.0").build(),
            Field::builder()
                .key("Content-Type")
                .value("text/plain")
                .build(),
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

    #[test]
    fn test_response_is_json() {
        let cases = vec![
            ("application/json", true),
            ("application/ld+json; charset=utf-8", true),
            ("text/json", true),
            ("application/vnd.github.raw+json", true),
            ("application/xml", false),
            ("application/atom+xml", false),
            ("image/jpeg", false),
        ];

        for (header, expected) in cases {
            // TODO: don't mind capitalization!
            let field = Field::builder().key("Content-Type").value(header).build();
            let table = FieldTable::from_iter(vec![field]);
            let response = Response::builder(&Request::default())
                .headers(&table)
                .build();
            assert_eq!(response.is_json(), expected);
        }
    }

    #[test]
    fn test_response_is_xml() {
        let cases = vec![
            ("application/json", false),
            ("application/ld+json; charset=utf-8", false),
            ("text/json", false),
            ("application/vnd.github.raw+json", false),
            ("application/xml", true),
            ("application/atom+xml", true),
            ("image/jpeg", false),
        ];

        for (header, expected) in cases {
            // TODO: don't mind capitalization!
            let field = Field::builder().key("Content-Type").value(header).build();
            let table = FieldTable::from_iter(vec![field]);
            let response = Response::builder(&Request::default())
                .headers(&table)
                .build();
            assert_eq!(response.is_xml(), expected);
        }
    }
}
