// Copyright 2024-2026 the Cartero authors
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
    use crate::{RequestBodyData, RequestBodyDataImpl};

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
        fn dup(&self) -> RequestBodyData {
            super::RequestBodyFile::builder()
                .path(self.obj().path())
                .content_type(self.obj().content_type())
                .build()
                .upcast()
        }

        fn body_type(&self) -> crate::RequestBodyType {
            crate::RequestBodyType::File
        }

        fn resolve(
            &self,
            tpl: &srtemplate::SrTemplate,
        ) -> Result<RequestBodyData, srtemplate::Error> {
            let path = tpl.render(&self.obj().path())?;
            let content_type = self
                .obj()
                .content_type()
                .map(|s| tpl.render(&s))
                .transpose()?;
            Ok(super::RequestBodyFile::builder()
                .path(&path)
                .content_type(content_type.as_ref())
                .build()
                .upcast())
        }

        fn rendered_headers(&self) -> Vec<(String, String)> {
            let content_type = self
                .obj()
                .content_type()
                .take_if(|str| !str.trim().is_empty())
                .unwrap_or("application/octet-stream".into());
            vec![("Content-Type".into(), content_type)]
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

        pub fn content_type(mut self, content_type: Option<impl AsRef<str>>) -> Self {
            self.builder = match content_type {
                Some(value) => self.builder.property("content-type", value.as_ref()),
                None => self.builder,
            };
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
    use srtemplate::SrTemplate;

    use crate::{utils::test::assert_emits_signal, RequestBodyDataExt};

    use super::*;

    #[test]
    fn test_resolve_with_content_type() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/{{DOCUMENT_ID}}/report.xml")
            .content_type(Some("application/{{FORMAT}}+xml"))
            .build();
        let template = SrTemplate::default();
        template.add_variable("DOCUMENT_ID", "1234");
        template.add_variable("FORMAT", "atom");

        let rendered_body = request_body
            .resolve(&template)
            .expect("Did not expect a fail")
            .downcast::<RequestBodyFile>()
            .expect("Did not downcast properly");

        assert_eq!(rendered_body.path(), "./assets/1234/report.xml");
        assert!(
            rendered_body
                .content_type()
                .is_some_and(|t| t == "application/atom+xml"),
            "content-type: {:?}",
            rendered_body.content_type()
        );
    }

    #[test]
    fn test_resolve_without_content_type() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/{{DOCUMENT_ID}}/report.xml")
            .build();
        let template = SrTemplate::default();
        template.add_variable("DOCUMENT_ID", "1234");

        let rendered_body = request_body
            .resolve(&template)
            .expect("Did not expect a fail")
            .downcast::<RequestBodyFile>()
            .expect("Did not downcast properly");
        assert_eq!(rendered_body.path(), "./assets/1234/report.xml");
        assert!(rendered_body.content_type().is_none());
    }

    #[test]
    #[should_panic]
    fn test_resolve_invalid_fails() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/{{DOCUMENT_ID}}/report.xml")
            .build();
        let template = SrTemplate::default();
        request_body.resolve(&template).unwrap();
    }

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
            .content_type(Some("application/xml"))
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

    #[test]
    fn test_renders_default_content_type() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/report.xml")
            .build();
        let headers = request_body.rendered_headers();
        assert_eq!(1, headers.len());
        assert_eq!(
            (
                "Content-Type".to_string(),
                "application/octet-stream".to_string()
            ),
            headers[0]
        );
    }

    #[test]
    fn test_renders_custom_content_type() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/report.xml")
            .content_type(Some("text/csv"))
            .build();
        let headers = request_body.rendered_headers();
        assert_eq!(1, headers.len());
        assert_eq!(
            ("Content-Type".to_string(), "text/csv".to_string()),
            headers[0]
        );
    }

    #[test]
    fn test_renders_empty_content_type() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/report.xml")
            .content_type(Some(""))
            .build();
        let headers = request_body.rendered_headers();
        assert_eq!(1, headers.len());
        assert_eq!(
            (
                "Content-Type".to_string(),
                "application/octet-stream".to_string()
            ),
            headers[0]
        );
    }
}
