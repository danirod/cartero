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

use glib::Object;
use glib::prelude::*;
use glib::subclass::prelude::*;

glib::wrapper! {
    /// Body payload with the request body encoded as provided.
    ///
    /// This is a more complex payload where the bytes that will be sent as
    /// part of an HTTP request are manually provided and encoded, and the
    /// semantics have to be provided by the user, usually encoding the
    /// payload as a JSON or XML document.
    ///
    /// A property to indicate the payload type is added, allowing to store
    /// these semantics as a field that can influentiate later the value of
    /// the `Content-Type` header when the request is issued.
    ///
    /// ## Properties
    ///
    /// - `payload`: the bytes object with the value that will be sent during
    ///   an HTTP request.
    /// - `payload-type`: a value that encodes the semantics of the payload.
    ///   This one is provided by the user and has values such as JSON, XML...
    ///   It is used both to set the default `Content-Type` header during
    ///   a request, and to choose the proper syntax highlighting in the text
    ///   editor used to set the payload.
    ///
    /// ## Setting up an instance
    ///
    /// - Use the `default` method to craft a new raw payload initialised to
    ///   an empty payload of type `octet-stream`.
    /// - Use the [`new`][RequestBodyRaw::new] method to craft a new payload,
    ///   specifiying both the payload and the payload type.
    pub struct RequestBodyRaw(ObjectSubclass<imp::RequestBodyRaw>) @extends crate::RequestBodyData;
}

impl Default for RequestBodyRaw {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl RequestBodyRaw {
    /// Create a new payload.
    ///
    /// The payload will be initialised to the type `raw_type`, and the given
    /// byte slice will be the initial contents of the payload data.
    pub fn new(raw_type: RequestBodyRawType, initial: impl AsRef<str>) -> Self {
        Object::builder()
            .property("payload-type", raw_type)
            .property("payload", initial.as_ref().to_owned())
            .build()
    }

    pub fn builder(payload_type: RequestBodyRawType) -> builder::RequestBodyRawBuilder {
        builder::RequestBodyRawBuilder::new(payload_type)
    }
}

/// Define the semantics of a [RequestBodyRaw] payload.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestBodyRawType")]
pub enum RequestBodyRawType {
    /// The payload has no semantics. By default this is linked to the
    /// application/octet-stream content-type, and it's the one that will be
    /// used when sending a request unless the user overrides the value.
    #[default]
    #[enum_value(name = "OCTET_STREAM", nick = "Octet Stream")]
    OctetStream,

    /// The payload should be treated as a JSON. The user interface may apply
    /// syntax highlighting in the text editor as if it was a JSON document.
    /// The default content type is application/json, unless overriden by the
    /// user.
    #[enum_value(name = "JSON", nick = "JSON")]
    Json,

    /// The payload should be treated as a XML. The user interface may apply
    /// syntax highlighting in the text editor as if it was a XML document.
    /// The default content type is application/xml, unless overriden by the
    /// user.
    #[enum_value(name = "XML", nick = "XML")]
    Xml,
}

mod imp {
    use std::cell::RefCell;

    use crate::{RequestBodyData, RequestBodyDataImpl};

    use super::*;
    use glib::Properties;

    use super::RequestBodyRawType;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestBodyRaw)]
    pub struct RequestBodyRaw {
        #[property(
            get,
            set,
            name = "payload-type",
            builder(RequestBodyRawType::OctetStream)
        )]
        payload_type: RefCell<RequestBodyRawType>,

        #[property(get, set)]
        payload: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyRaw {
        const NAME: &'static str = "CarteroRequestBodyRaw";
        type Type = super::RequestBodyRaw;
        type ParentType = crate::RequestBodyData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBodyRaw {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_payload_notify(|raw| {
                raw.emit_by_name::<()>("changed", &[&"payload"]);
            });
            self.obj().connect_payload_type_notify(|raw| {
                raw.emit_by_name::<()>("changed", &[&"payload-type"]);
            });
        }
    }

    impl RequestBodyDataImpl for RequestBodyRaw {
        fn dup(&self) -> RequestBodyData {
            super::RequestBodyRaw::builder(self.obj().payload_type())
                .payload(self.obj().payload())
                .build()
                .upcast()
        }

        fn body_type(&self) -> crate::RequestBodyType {
            crate::RequestBodyType::Raw
        }

        fn resolve(
            &self,
            tpl: &srtemplate::SrTemplate,
        ) -> Result<RequestBodyData, srtemplate::Error> {
            let payload = tpl.render(self.obj().payload())?;
            let payload_type = self.obj().payload_type();
            Ok(super::RequestBodyRaw::builder(payload_type)
                .payload(payload)
                .build()
                .upcast())
        }

        fn rendered_headers(&self) -> Vec<(String, String)> {
            let content_type = match self.obj().payload_type() {
                RequestBodyRawType::Json => "application/json",
                RequestBodyRawType::OctetStream => "application/octet-stream",
                RequestBodyRawType::Xml => "application/xml",
            };
            vec![("Content-Type".into(), content_type.into())]
        }
    }
}

mod builder {
    use glib::object::ObjectBuilder;

    use super::*;

    pub struct RequestBodyRawBuilder {
        builder: ObjectBuilder<'static, RequestBodyRaw>,
    }

    impl RequestBodyRawBuilder {
        pub fn new(raw_type: RequestBodyRawType) -> Self {
            Self {
                builder: glib::Object::builder().property("payload-type", raw_type),
            }
        }

        pub fn build(self) -> RequestBodyRaw {
            self.builder.build()
        }

        pub fn payload(mut self, payload: impl AsRef<str>) -> Self {
            self.builder = self.builder.property("payload", payload.as_ref());
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use glib::object::Cast;
    use srtemplate::SrTemplate;

    use crate::{
        RequestBodyDataExt, RequestBodyRawType, RequestBodyType, utils::test::assert_emits_signal,
    };

    use super::RequestBodyRaw;

    #[test]
    fn test_builder() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::Xml)
            .payload(r#"<?xml version="1.0" ?><document />"#)
            .build();
        assert_eq!(raw.payload_type(), RequestBodyRawType::Xml);
        assert_eq!(raw.payload(), r#"<?xml version="1.0" ?><document />"#);
    }

    #[test]
    fn test_defaults() {
        let raw = RequestBodyRaw::default();
        assert_eq!(raw.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(raw.payload().len(), 0);
    }

    #[test]
    fn test_new() {
        let raw = RequestBodyRaw::new(RequestBodyRawType::Xml, "<?xml?>");
        assert_eq!(raw.payload_type(), RequestBodyRawType::Xml);
        assert_eq!(raw.payload(), "<?xml?>");
    }

    #[test]
    pub fn test_change_type() {
        let raw = RequestBodyRaw::default();
        assert_emits_signal(&raw, "notify::payload-type", || {
            raw.set_payload_type(RequestBodyRawType::Json)
        });
        assert_eq!(RequestBodyRawType::Json, raw.payload_type());
    }

    #[test]
    pub fn test_change_payload() {
        let raw = RequestBodyRaw::default();
        assert_emits_signal(&raw, "notify::payload", || {
            raw.set_payload("hello world");
        });
        assert_eq!(11, raw.payload().len());
    }

    #[test]
    pub fn test_body_type() {
        let body: RequestBodyRaw = RequestBodyRaw::default();
        assert_eq!(body.body_type(), RequestBodyType::Raw);
    }

    #[test]
    pub fn test_resolve_successful() {
        let raw = RequestBodyRaw::new(RequestBodyRawType::OctetStream, "hello {{WHO}}");
        let tpl = SrTemplate::default();
        tpl.add_variable("WHO", "world");
        let resolved_raw = raw
            .resolve(&tpl)
            .expect("Invalid resolve")
            .downcast::<RequestBodyRaw>()
            .expect("Invalid downcast");
        assert_eq!(resolved_raw.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(resolved_raw.payload(), "hello world");
    }

    #[test]
    #[should_panic]
    pub fn test_resolve_unsuccessful() {
        let raw = RequestBodyRaw::new(RequestBodyRawType::OctetStream, "hello {{WHO}}");
        let tpl = SrTemplate::default();
        raw.resolve(&tpl).expect("Invalid resolve");
    }

    #[test]
    pub fn test_emits_signals() {
        let body = RequestBodyRaw::builder(RequestBodyRawType::Json).build();
        assert_emits_signal(&body, "changed", || body.set_payload("hello"));
        assert_emits_signal(&body, "changed", || {
            body.set_payload_type(RequestBodyRawType::Xml)
        });
    }

    #[test]
    fn test_rendered_headers_on_json() {
        let body = RequestBodyRaw::new(RequestBodyRawType::Json, r#"{"hello": "world"}"#);
        let headers = body.rendered_headers();
        assert_eq!(1, headers.len());
        assert_eq!(
            ("Content-Type".into(), "application/json".into()),
            headers[0]
        );
    }

    #[test]
    fn test_rendered_headers_on_xml() {
        let body =
            RequestBodyRaw::new(RequestBodyRawType::Xml, r#"<?xml version="1.0" ?><data />"#);
        let headers = body.rendered_headers();
        assert_eq!(1, headers.len());
        assert_eq!(
            ("Content-Type".into(), "application/xml".into()),
            headers[0]
        );
    }

    #[test]
    fn test_rendered_headers_on_octet_stream() {
        let body = RequestBodyRaw::new(RequestBodyRawType::OctetStream, r#"hello world"#);
        let headers = body.rendered_headers();
        assert_eq!(1, headers.len());
        assert_eq!(
            ("Content-Type".into(), "application/octet-stream".into()),
            headers[0]
        );
    }
}
