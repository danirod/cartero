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
use glib::Object;

glib::wrapper! {
    /// Body payload that is read from a file.
    ///
    /// When resolved, this body will read the contents of the file behind the given path and set
    /// them as request body. Optionally, the content type may be given.
    ///
    /// This is similar to using `RequestBodyRaw` and pasting the contents of the file as the
    /// payload to send. Or it's like using raw payloads, except that the contents is indirectly
    /// stored in another file.
    ///
    /// For security reasons, the client may reject reading file contents from paths in special or
    /// protected locations, or limit to just a few valid directories. For instance, a path maybe
    /// is only valid if it's in the same directory as the request file or a subdirectory.
    ///
    /// The content type can be inferred in runtime with the proper tools (such as MIME type
    /// guessing), or set by the user.
    ///
    /// ## Properties
    ///
    /// - `path`: the path to the file to get the content from.
    /// - `content_type`: an optional value for the Content-Type header
    pub struct RequestBodyFile(ObjectSubclass<imp::RequestBodyFile>) @extends crate::RequestBodyData;
}

impl Default for RequestBodyFile {
    fn default() -> Self {
        Object::new()
    }
}

impl RequestBodyFile {
    pub fn new(path: impl AsRef<str>, content_type: Option<impl ToString>) -> Self {
        Object::builder()
            .property("path", path.as_ref())
            .property_if_some("content-type", content_type.map(|c| c.to_string()))
            .build()
    }

    pub fn builder() -> builder::RequestBodyFileBuilder {
        builder::RequestBodyFileBuilder::new()
    }
}

mod imp {
    use crate::RequestBodyDataImpl;

    use super::*;
    use glib::Properties;
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestBodyFile)]
    pub struct RequestBodyFile {
        #[property(get, set)]
        path: RefCell<String>,

        // application/octet-stream by default
        #[property(get, set, nullable)]
        content_type: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyFile {
        const NAME: &'static str = "CarteroRequestBodyFile";
        type Type = super::RequestBodyFile;
        type ParentType = crate::RequestBodyData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBodyFile {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_path_notify(|file| {
                file.emit_by_name::<()>("changed", &[&"file"]);
            });
            self.obj().connect_content_type_notify(|file| {
                file.emit_by_name::<()>("changed", &[&"content-type"]);
            });
        }
    }

    impl RequestBodyDataImpl for RequestBodyFile {
        fn body_type(&self) -> crate::RequestBodyType {
            crate::RequestBodyType::File
        }
    }
}

mod builder {
    use super::*;
    use glib::object::ObjectBuilder;

    pub struct RequestBodyFileBuilder {
        builder: ObjectBuilder<'static, RequestBodyFile>,
    }

    impl RequestBodyFileBuilder {
        pub fn new() -> Self {
            Self {
                builder: glib::Object::builder(),
            }
        }

        pub fn content_type(mut self, content_type: impl AsRef<str>) -> Self {
            self.builder = self.builder.property("content-type", content_type.as_ref());
            self
        }

        pub fn path(mut self, content_type: impl AsRef<str>) -> Self {
            self.builder = self.builder.property("path", content_type.as_ref());
            self
        }

        pub fn build(self) -> RequestBodyFile {
            self.builder.build()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test::assert_emits_signal;

    use super::*;

    #[test]
    fn test_builder_simple_case() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/testing.png")
            .build();
        assert_eq!(request_body.path(), "./assets/testing.png");
        assert!(request_body.content_type().is_none());
    }

    #[test]
    fn test_builder_with_content_type() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/report.xml")
            .content_type("application/xml")
            .build();
        assert_eq!(request_body.path(), "./assets/report.xml");
        assert!(request_body
            .content_type()
            .is_some_and(|content_type| content_type == "application/xml"));
    }

    #[test]
    fn test_emits_signals() {
        let body = RequestBodyFile::builder().build();
        assert_emits_signal(&body, "changed", || {
            body.set_content_type(Some("application/xml"));
        });
        assert_emits_signal(&body, "changed", || body.set_path("report.xml"));
    }
}
