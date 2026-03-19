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

use glib::subclass::prelude::*;
use glib::{Object, prelude::*};
use srtemplate::SrTemplate;

use crate::{
    RequestBodyData, RequestBodyDataExt, RequestBodyFile, RequestBodyMultipart, RequestBodyRaw,
    RequestBodyUrlencoded,
};

glib::wrapper! {
    /// A special object to assign payload data to a request.
    ///
    /// This object is meant to be decoded when issuing an HTTP request in
    /// order to build the request body that is sent with the request, if
    /// applicable. The user interface may present widgets and forms in order
    /// to let the user change the payload stored in a request. The body
    /// may be saved to disk when persisting a request, or exported to
    /// different formats.
    ///
    /// ## Body data and body type
    ///
    /// There are different kinds of bodies. Each kind of payload is bound to
    /// a **type**, backed by one of the variants of [RequestBodyType][super::RequestBodyType].
    /// You can use the `body-type` property of a `RequestBody` to get the
    /// current type of payload in use.
    ///
    /// Most of the types are then bound to a specific **data**, which is
    /// the actual payload attached to the request. Each body type provides a
    /// custom body data class, with its custom strategy on how to assign the
    /// body of a request, and params to get or set the contents of the payload.
    ///
    /// Each one of these datas is a subclass of [RequestBodyData][super::RequestBodyData].
    ///
    /// | Type | Data |
    /// | ---- | ---- |
    /// | [`None`][RequestBodyType::None] | (none) |
    /// | [`UrlEncoded`][RequestBodyType::UrlEncoded] | [RequestBodyUrlencoded][super::RequestBodyUrlencoded] |
    /// | [`Multipart`][RequestBodyType::Multipart] | [RequestBodyMultipart][super::RequestBodyMultipart] |
    /// | [`Raw`][RequestBodyType::Raw] | [RequestBodyRaw][super::RequestBodyRaw] |
    /// | [`File`][RequestBodyType::File] | [RequestBodyFile][super::RequestBodyFile] |
    ///
    /// Note that the `body-data` property is of type `RequestBodyData`. You will
    /// have to downcast using the `.downcast()` and `.and_downcast()` methods
    /// in order to get a reference of the proper type if you want to read or
    /// update the contents of the payload. There are methods in a `RequestBody`
    /// to help with this.
    ///
    /// ## Parameters
    ///
    /// - `body-data`: the body data object that defines the payload of a request.
    /// - `body-type`: the type of payload in use for this `RequestBody`.
    ///
    /// ## Creating a Body object
    ///
    /// There are two ways to create a RequestBody:
    ///
    /// - Use the `default()` method to create a new `RequestBody` object with
    ///   the payload set to None. This is an empty object, and it is treated
    ///   as a noop when creating an HTTP request, without sending an actual
    ///   body. Most exporters wouldn't even set the payload when saving or
    ///   exporting the request into a file.
    ///
    /// ```
    /// use cartero_objects::{RequestBody, RequestBodyType};
    ///
    /// let body = RequestBody::default();
    /// assert_eq!(body.body_type(), RequestBodyType::None);
    /// assert!(body.body_data().is_none());
    /// ```
    ///
    /// - Use the `new()` method to initialise the body with an initial type
    ///   and optionally some initial payload. This payload can be `None` to
    ///   set it to the default value for that specific type.
    ///
    /// ```
    /// use cartero_objects::*;
    ///
    /// // Create a new body of type urlencoded, with empty parameters.
    /// let urlencoded = RequestBody::new(RequestBodyType::UrlEncoded, RequestBodyData::NONE);
    /// assert_eq!(urlencoded.body_type(), RequestBodyType::UrlEncoded);
    /// assert!(urlencoded.body_data().is_some_and(|d| d.body_type() == RequestBodyType::UrlEncoded));
    ///
    /// let data = r#"{"error": true, "detail": "User not found"}"#;
    /// let raw = RequestBodyRaw::new(RequestBodyRawType::Json, data);
    /// let body = RequestBody::new(RequestBodyType::Raw, Some(raw));
    /// assert_eq!(body.body_type(), RequestBodyType::Raw);
    /// assert!(body.body_data().is_some_and(|d| d.body_type() == RequestBodyType::Raw));
    /// ```
    ///
    /// ## Type checking and restrictions
    ///
    /// Some body types require a specific class of body data. Therefore, the
    /// setters in `RequestBody` will do their best to assert that the
    /// `body-data` of a `RequestBody` is always compatible with the
    /// `body-type`.
    ///
    /// - When the `body-type` of a `RequestBody` changes, the `body-data` is
    ///   also reset to an empty object associated to the type of the new
    ///   `body-type`. Therefore, the following shall be verified:
    ///
    /// ```
    /// use cartero_objects::*;
    /// use glib::object::ObjectExt;
    ///
    /// // Start with a request body payload of type Urlencoded.
    /// let body = RequestBody::new(RequestBodyType::UrlEncoded, RequestBodyData::NONE);
    ///
    /// // Therefore, the body-data is currently for UrlEncoded.
    /// assert!(body.body_data().is_some_and(|d| d.is::<RequestBodyUrlencoded>()));
    ///
    /// // But if we change the type...
    /// body.set_body_type(RequestBodyType::Multipart);
    ///
    /// // ...the body-data changes to a new type too.
    /// assert!(body.body_data().is_some_and(|d| d.is::<RequestBodyMultipart>()));
    /// ```
    ///
    /// - If the `body-data` changes to a different object, type checking will
    ///   be done to make sure that the given `body-data` is compatible with
    ///   the current `body-type`.- If it's not, then the value will not
    ///   actually change. **I wish I could make the setter panic**, but+
    ///   unfortunately it seems that gtk-rs does not support so (or I couldn't
    ///   manage to panic and successfully unwind the panic). However, the
    ///   `notify::body-data` signal is not emitted.
    ///
    /// ```
    /// use cartero_objects::*;
    /// use glib::object::CastNone;
    /// use gio::prelude::ListModelExt;
    ///
    /// // Start with a specific payload.
    /// let urlencoded = RequestBodyUrlencoded::default();
    /// let body = RequestBody::new(RequestBodyType::UrlEncoded, Some(urlencoded));
    /// assert_eq!(body.body_data().and_downcast::<RequestBodyUrlencoded>().unwrap().params().n_items(), 0);
    ///
    /// // We can change to a new object of the same type.
    /// let urlencoded2 = RequestBodyUrlencoded::default();
    /// urlencoded2.params().insert(&Field::from(("user_id", "1000")));
    /// body.set_body_data(Some(urlencoded2));
    /// assert_eq!(body.body_data().and_downcast::<RequestBodyUrlencoded>().unwrap().params().n_items(), 1);
    ///
    /// // However, you cannot just change to a different class.
    /// let raw = RequestBodyRaw::default();
    /// body.set_body_data(Some(raw));
    /// assert_eq!(body.body_data().unwrap().body_type(), RequestBodyType::UrlEncoded)
    /// ```
    pub struct RequestBody(ObjectSubclass<imp::RequestBody>);
}

impl RequestBody {
    pub fn builder() -> builder::RequestBodyBuilder {
        builder::RequestBodyBuilder::default()
    }

    pub(crate) fn resolve(&self, processor: &SrTemplate) -> Result<Self, srtemplate::Error> {
        let data = self.body_data().map(|b| b.resolve(processor)).transpose()?;
        Ok(Self::new(self.body_type(), data))
    }
}

impl Default for RequestBody {
    fn default() -> Self {
        Object::builder().build()
    }
}

fn default_body_data(body_type: RequestBodyType) -> Option<RequestBodyData> {
    match body_type {
        RequestBodyType::UrlEncoded => Some(RequestBodyUrlencoded::default().upcast()),
        RequestBodyType::Multipart => Some(RequestBodyMultipart::default().upcast()),
        RequestBodyType::Raw => Some(RequestBodyRaw::default().upcast()),
        RequestBodyType::File => Some(RequestBodyFile::default().upcast()),
        _ => None,
    }
}

impl RequestBody {
    /// Create a new request body with the given initial values.
    ///
    /// The `body_type` variant specifies the initial type of payload, and
    /// optionally a `body_data` can be given with the actual payload of that
    /// specific kind.
    pub fn new<T>(body_type: RequestBodyType, body_data: Option<T>) -> Self
    where
        T: IsA<RequestBodyData>,
    {
        let body_data: Option<RequestBodyData> = body_data
            .map(|data| data.upcast())
            .or_else(|| default_body_data(body_type));
        Object::builder()
            .property("body-type", body_type)
            .property("body-data", body_data)
            .build()
    }

    pub fn dup(&self) -> Self {
        Self::new(self.body_type(), self.body_data().map(|body| body.dup()))
    }

    /// Returns the urlencoded payload, if the body is of such type.
    pub fn urlencoded(&self) -> Option<RequestBodyUrlencoded> {
        if self.body_type() == RequestBodyType::UrlEncoded {
            self.body_data().and_downcast::<RequestBodyUrlencoded>()
        } else {
            None
        }
    }

    /// Returns the multipart payload, if the body is of such type.
    pub fn multipart(&self) -> Option<RequestBodyMultipart> {
        if self.body_type() == RequestBodyType::Multipart {
            self.body_data().and_downcast::<RequestBodyMultipart>()
        } else {
            None
        }
    }

    /// Returns the raw payload, if the body is of such type.
    pub fn raw(&self) -> Option<RequestBodyRaw> {
        if self.body_type() == RequestBodyType::Raw {
            self.body_data().and_downcast::<RequestBodyRaw>()
        } else {
            None
        }
    }

    /// Returns the file payload, if the body is of such type.
    pub fn file(&self) -> Option<RequestBodyFile> {
        if self.body_type() == RequestBodyType::File {
            self.body_data().and_downcast::<RequestBodyFile>()
        } else {
            None
        }
    }
}

/// The kind of body being used in a [RequestBody] form.
///
/// There are different kinds of bodies, and this enum allows both the
/// user interface and the interoperabilityh libraries know which one is
/// the one in use.
///
/// Changing the type of a `RequestBody` may also change the state of the
/// user interface or form, in order to accomodate for the different inputs
/// required by the changed type.
///
/// Please see the doc for [RequestBody] to know more about the relationship
/// between this enum and the form object.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestBodyType")]
pub enum RequestBodyType {
    /// No body is set.
    #[default]
    #[enum_value(name = "NONE", nick = "None")]
    None,

    /// The body is a set of url-encoded key value pairs.
    #[enum_value(name = "URL_ENCODED", nick = "URL-Encoded")]
    UrlEncoded,

    /// The body is encoded as a multipart/form-data object.
    #[enum_value(name = "MULTIPART", nick = "Multipart")]
    Multipart,

    /// The body is encoded in raw: the bytes of the payload will be provided
    /// and maybe the content type will be infered from the `Content-Type`
    /// header.
    #[enum_value(name = "RAW", nick = "Raw")]
    Raw,

    // The body is stored in a file, and during request time it is loaded
    // by the HTTP client or whatever application is using the structure.
    #[enum_value(name = "FILE", nick = "File")]
    File,
}

mod builder {
    use glib::object::ObjectBuilder;

    use crate::RequestBodyFile;

    use super::*;

    pub struct RequestBodyBuilder {
        builder: ObjectBuilder<'static, RequestBody>,
    }

    impl RequestBodyBuilder {
        pub fn default() -> Self {
            Self {
                builder: Object::builder(),
            }
        }

        pub fn build(self) -> RequestBody {
            self.builder.build()
        }

        pub fn none(mut self) -> Self {
            self.builder = self.builder.property("body-type", RequestBodyType::None);
            self
        }

        pub fn urlencoded(mut self, url: &RequestBodyUrlencoded) -> Self {
            self.builder = self
                .builder
                .property("body-type", RequestBodyType::UrlEncoded)
                .property("body-data", url);
            self
        }

        pub fn multipart(mut self, mp: &RequestBodyMultipart) -> Self {
            self.builder = self
                .builder
                .property("body-type", RequestBodyType::Multipart)
                .property("body-data", mp);
            self
        }

        pub fn raw(mut self, raw: &RequestBodyRaw) -> Self {
            self.builder = self
                .builder
                .property("body-type", RequestBodyType::Raw)
                .property("body-data", raw);
            self
        }

        pub fn file(mut self, file: &RequestBodyFile) -> Self {
            self.builder = self
                .builder
                .property("body-type", RequestBodyType::File)
                .property("body-data", file);
            self
        }
    }
}

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        sync::OnceLock,
    };

    use glib::{Properties, SignalGroup, subclass::Signal};

    use crate::{RequestBodyData, RequestBodyDataExt};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestBody)]
    pub struct RequestBody {
        #[property(get, set = Self::set_body_type, name = "body-type", builder(RequestBodyType::None))]
        body_type: RefCell<RequestBodyType>,

        #[property(get, set = Self::set_body_data, explicit_notify, name = "body-data", nullable)]
        body_data: RefCell<Option<RequestBodyData>>,
        body_data_group: OnceCell<SignalGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBody {
        const NAME: &'static str = "CarteroRequestBody";
        type Type = super::RequestBody;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBody {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_signal_group();

            self.obj().connect_body_type_notify(|auth| {
                auth.emit_by_name::<()>("changed", &[&"type"]);
            });
            self.obj().connect_body_data_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |body| {
                    imp.body_data_group
                        .get()
                        .unwrap()
                        .set_target(body.body_data().as_ref());
                    body.emit_by_name::<()>("changed", &[&"data"]);
                }
            ));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("changed")
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl RequestBody {
        fn init_signal_group(&self) {
            let obj = self.obj();

            let body_data_group = SignalGroup::new::<RequestBodyData>();
            body_data_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &RequestBodyData, param: &str| {
                        let param = format!("data.{param}");
                        obj.emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            self.body_data_group.set(body_data_group).unwrap();
        }

        fn set_body_type(&self, body_type: RequestBodyType) {
            if *self.body_type.borrow() == body_type {
                return;
            }
            let next = default_body_data(body_type);
            self.body_type.replace(body_type);
            self.obj().set_body_data(next);
            self.obj().notify_body_data();
        }

        fn set_body_data(&self, body_data: Option<RequestBodyData>) {
            let current_type = self.obj().body_type();
            let valid = match current_type {
                RequestBodyType::None => body_data.is_none(),
                _ => body_data
                    .as_ref()
                    .is_some_and(|data| data.body_type() == current_type),
            };
            if valid {
                self.body_data.replace(body_data);
                self.obj().notify_body_data();
            } else {
                #[cfg(not(test))]
                glib::g_critical!(
                    "Cartero",
                    "set_body_data() was called with a RequestBodyData of invalid RequestBodyType for this RequestBody object"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use gio::prelude::ListModelExt;
    use glib::object::CastNone;
    use srtemplate::SrTemplate;

    use crate::{
        Field, FieldTable, RequestBodyData, RequestBodyDataExt, RequestBodyFile,
        RequestBodyMultipart, RequestBodyRaw, RequestBodyRawType, RequestBodyType,
        RequestBodyUrlencoded,
        utils::test::{assert_emits_signal, assert_emits_signals, assert_not_emits_signal},
    };

    use super::RequestBody;

    #[test]
    pub fn new_for_none() {
        let body = RequestBody::new(super::RequestBodyType::None, RequestBodyData::NONE);
        assert_eq!(RequestBodyType::None, body.body_type());
        assert!(body.body_data().is_none());
    }

    #[test]
    pub fn new_for_urlencoded_with_default() {
        let body = RequestBody::new(RequestBodyType::UrlEncoded, RequestBodyData::NONE);
        assert_eq!(RequestBodyType::UrlEncoded, body.body_type());
        let body_data = body
            .body_data()
            .and_downcast::<RequestBodyUrlencoded>()
            .unwrap();
        assert_eq!(0, body_data.params().n_items());
    }

    #[test]
    pub fn new_for_urlencoded_with_initial() {
        let field = Field::from(("user_id", "1000"));
        let table = FieldTable::from_iter([field]);
        let payload = RequestBodyUrlencoded::from_table(&table);
        let body = RequestBody::new(RequestBodyType::UrlEncoded, Some(payload));
        assert_eq!(RequestBodyType::UrlEncoded, body.body_type());
        let body_data = body
            .body_data()
            .and_downcast::<RequestBodyUrlencoded>()
            .unwrap();
        assert_eq!(1, body_data.params().n_items());
    }

    #[test]
    pub fn new_for_multipart_with_default() {
        let body = RequestBody::new(RequestBodyType::Multipart, RequestBodyData::NONE);
        assert_eq!(RequestBodyType::Multipart, body.body_type());
        let body_data = body
            .body_data()
            .and_downcast::<RequestBodyMultipart>()
            .unwrap();
        assert_eq!(0, body_data.params().n_items());
    }

    #[test]
    pub fn new_for_multipart_with_initial() {
        let field = Field::from(("user_id", "1000"));
        let table = FieldTable::from_iter([field]);
        let payload = RequestBodyMultipart::from_table(&table);
        let body = RequestBody::new(RequestBodyType::Multipart, Some(payload));
        assert_eq!(RequestBodyType::Multipart, body.body_type());
        let body_data = body
            .body_data()
            .and_downcast::<RequestBodyMultipart>()
            .unwrap();
        assert_eq!(1, body_data.params().n_items());
    }

    #[test]
    pub fn new_for_raw_with_default() {
        let body = RequestBody::new(RequestBodyType::Raw, RequestBodyData::NONE);
        assert_eq!(RequestBodyType::Raw, body.body_type());
        let body_data = body.body_data().and_downcast::<RequestBodyRaw>().unwrap();
        assert_eq!(body_data.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(body_data.payload().len(), 0);
    }

    #[test]
    pub fn new_for_raw_with_initial() {
        let payload = RequestBodyRaw::new(RequestBodyRawType::Json, "[1, 2, 4]");
        let body = RequestBody::new(RequestBodyType::Raw, Some(payload));
        assert_eq!(RequestBodyType::Raw, body.body_type());
        let body_data = body.body_data().and_downcast::<RequestBodyRaw>().unwrap();
        assert_eq!(body_data.payload_type(), RequestBodyRawType::Json);
        assert_eq!(body_data.payload().len(), 9);
    }

    #[test]
    pub fn new_for_file_with_default() {
        let payload = RequestBodyFile::default();
        let body = RequestBody::new(RequestBodyType::File, Some(payload));
        assert_eq!(body.body_type(), RequestBodyType::File);
        let data = body
            .body_data()
            .and_downcast::<RequestBodyFile>()
            .expect("No body of type file?");
        assert_eq!(data.path(), "");
        assert!(data.content_type().is_none());
    }

    #[test]
    pub fn new_for_file_with_initial() {
        let payload = RequestBodyFile::new("assets/report.xml", Some("application/xml"));
        let body = RequestBody::new(RequestBodyType::File, Some(payload));
        assert_eq!(body.body_type(), RequestBodyType::File);
        let data = body
            .body_data()
            .and_downcast::<RequestBodyFile>()
            .expect("No body of type file?");
        assert_eq!(data.path(), "assets/report.xml");
        assert!(data.content_type().is_some_and(|f| f == "application/xml"));
    }

    #[test]
    pub fn set_body_type_changes_data_type() {
        let body = RequestBody::new(RequestBodyType::UrlEncoded, RequestBodyData::NONE);
        assert!(
            body.body_data()
                .is_some_and(|data| data.body_type() == RequestBodyType::UrlEncoded)
        );
        assert_emits_signals(&body, &["notify::body-type", "notify::body-data"], || {
            body.set_body_type(RequestBodyType::Multipart)
        });
        assert!(
            body.body_data()
                .is_some_and(|data| data.body_type() == RequestBodyType::Multipart)
        );
    }

    #[test]
    pub fn set_body_data_with_same_type() {
        let body = RequestBody::new(RequestBodyType::UrlEncoded, RequestBodyData::NONE);
        let payload = RequestBodyUrlencoded::new();
        assert_emits_signal(&body, "notify::body-data", || {
            body.set_body_data(Some(payload.as_ref()))
        });
    }

    #[test]
    pub fn set_body_data_with_distinct_type() {
        let body = RequestBody::new(RequestBodyType::Multipart, RequestBodyData::NONE);
        let payload = RequestBodyUrlencoded::new();
        assert_not_emits_signal(&body, "notify::body-data", || {
            body.set_body_data(Some(payload.as_ref()))
        });
    }

    #[test]
    pub fn builder_default() {
        let body = RequestBody::builder().build();
        assert_eq!(body.body_type(), RequestBodyType::None);
        assert!(body.body_data().is_none());
    }

    #[test]
    pub fn builder_urlencoded() {
        let urlencoded = RequestBodyUrlencoded::builder()
            .field(&Field::builder().key("user_id").value("1").build())
            .build();
        let body = RequestBody::builder().urlencoded(&urlencoded).build();
        assert_eq!(body.body_type(), RequestBodyType::UrlEncoded);
        let data = body.urlencoded().unwrap();
        assert_eq!(1, data.params().n_items());
    }

    #[test]
    pub fn builder_multipart() {
        let multipart = RequestBodyMultipart::builder()
            .field(&Field::builder().key("user_id").value("1").build())
            .build();
        let body = RequestBody::builder().multipart(&multipart).build();
        assert_eq!(body.body_type(), RequestBodyType::Multipart);
        let data = body.multipart().unwrap();
        assert_eq!(1, data.params().n_items());
    }

    #[test]
    pub fn test_raw() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
            .payload("hello world")
            .build();
        let body = RequestBody::builder().raw(&raw).build();
        assert_eq!(body.body_type(), RequestBodyType::Raw);
        let data = body.raw().unwrap();
        assert_eq!(data.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(data.payload(), "hello world");
    }

    #[test]
    pub fn test_file() {
        let file = RequestBodyFile::builder().path("assets/report.xml").build();
        let body = RequestBody::builder().file(&file).build();
        assert_eq!(body.body_type(), RequestBodyType::File);
        let data = body.file().unwrap();
        assert_eq!(data.path(), "assets/report.xml");
        assert!(data.content_type().is_none());
    }

    #[test]
    pub fn test_emits_signal_on_change_none() {
        let body = RequestBody::new(RequestBodyType::None, RequestBodyData::NONE);
        assert_emits_signal(&body, "changed", || {
            body.set_body_type(RequestBodyType::Multipart);
        });
    }

    #[test]
    pub fn test_emits_signal_on_multipart_change() {
        let multipart = RequestBodyMultipart::builder()
            .field(&Field::builder().key("user_id").value("1").build())
            .build();
        let body = RequestBody::builder().multipart(&multipart).build();

        assert_emits_signal(&body, "changed", || {
            body.multipart()
                .expect("This is not a multipart")
                .params()
                .insert(&Field::builder().build());
        });
        assert_emits_signal(&body, "changed", || {
            body.multipart()
                .expect("This is not a multipart")
                .set_params(FieldTable::default());
        });
        assert_emits_signal(&body, "changed", || {
            body.multipart()
                .expect("This is not a multipart")
                .params()
                .insert(&Field::builder().build());
        });
    }

    #[test]
    pub fn test_emits_signal_on_urlencoded_change() {
        let urlencoded = RequestBodyUrlencoded::builder()
            .field(&Field::builder().key("user_id").value("1").build())
            .build();
        let body = RequestBody::builder().urlencoded(&urlencoded).build();

        assert_emits_signal(&body, "changed", || {
            body.urlencoded()
                .expect("This is not a urlencoded")
                .params()
                .insert(&Field::builder().build());
        });
        assert_emits_signal(&body, "changed", || {
            body.urlencoded()
                .expect("This is not a urlencoded")
                .set_params(FieldTable::default());
        });
        assert_emits_signal(&body, "changed", || {
            body.urlencoded()
                .expect("This is not a urlencoded")
                .params()
                .insert(&Field::builder().build());
        });
    }

    #[test]
    pub fn test_emits_signal_on_raw_change() {
        let body = RequestBody::new(RequestBodyType::Raw, RequestBodyData::NONE);

        assert_emits_signal(&body, "changed", || {
            body.raw().expect("This is not raw").set_payload("hello");
        });
        assert_emits_signal(&body, "changed", || {
            body.raw()
                .expect("This is not raw")
                .set_payload_type(RequestBodyRawType::Xml);
        });
    }

    #[test]
    pub fn test_emits_signal_on_file_change() {
        let body = RequestBody::new(RequestBodyType::File, RequestBodyData::NONE);

        assert_emits_signal(&body, "changed", || {
            body.file()
                .expect("This is not file")
                .set_path("assets/report.xml");
        });
        assert_emits_signal(&body, "changed", || {
            body.file()
                .expect("This is not file")
                .set_content_type(Some("application/xml"));
        });
    }

    #[test]
    pub fn test_resolve_body_with_none() {
        let body = RequestBody::builder().none().build();
        let tpl = SrTemplate::default();
        let resolved_body = body.resolve(&tpl).expect("Invalid resolve");
        assert!(resolved_body.body_data().is_none());
        assert_eq!(resolved_body.body_type(), RequestBodyType::None);
    }

    #[test]
    pub fn test_resolve_body_with_urlencoded() {
        let field1 = Field::builder()
            .key("User-Agent")
            .value("{{USER_AGENT}}")
            .build();
        let field2 = Field::builder()
            .key("Accept")
            .value("application/json")
            .build();
        let body = RequestBodyUrlencoded::builder()
            .field(&field1)
            .field(&field2)
            .build();
        let body = RequestBody::builder().urlencoded(&body).build();
        let tpl = SrTemplate::default();
        tpl.add_variable("USER_AGENT", "Mozilla/5.0");
        let resolved_body = body.resolve(&tpl).expect("Invalid resolve");
        assert_eq!(resolved_body.body_type(), RequestBodyType::UrlEncoded);
        let resolved_urlencoded = resolved_body.urlencoded().expect("Is not urlencoded?");
        assert_eq!(2, resolved_urlencoded.params().n_items());
        assert_eq!(
            "User-Agent",
            resolved_urlencoded.params().field(0).unwrap().key()
        );
        assert_eq!(
            "Mozilla/5.0",
            resolved_urlencoded.params().field(0).unwrap().value()
        );
        assert_eq!(
            "Accept",
            resolved_urlencoded.params().field(1).unwrap().key()
        );
        assert_eq!(
            "application/json",
            resolved_urlencoded.params().field(1).unwrap().value()
        );
    }

    #[test]
    pub fn test_resolve_body_with_multipart() {
        let field1 = Field::builder()
            .key("User-Agent")
            .value("{{USER_AGENT}}")
            .build();
        let field2 = Field::builder()
            .key("Accept")
            .value("application/json")
            .build();
        let body = RequestBodyMultipart::builder()
            .field(&field1)
            .field(&field2)
            .build();
        let body = RequestBody::builder().multipart(&body).build();
        let tpl = SrTemplate::default();
        tpl.add_variable("USER_AGENT", "Mozilla/5.0");
        let resolved_body = body.resolve(&tpl).expect("Invalid resolve");
        assert_eq!(resolved_body.body_type(), RequestBodyType::Multipart);
        let resolved_multipart = resolved_body.multipart().expect("Is not multipart?");
        assert_eq!(2, resolved_multipart.params().n_items());
        assert_eq!(
            "User-Agent",
            resolved_multipart.params().field(0).unwrap().key()
        );
        assert_eq!(
            "Mozilla/5.0",
            resolved_multipart.params().field(0).unwrap().value()
        );
        assert_eq!(
            "Accept",
            resolved_multipart.params().field(1).unwrap().key()
        );
        assert_eq!(
            "application/json",
            resolved_multipart.params().field(1).unwrap().value()
        );
    }

    #[test]
    fn test_resolve_body_with_raw() {
        let raw = RequestBodyRaw::new(RequestBodyRawType::OctetStream, "hello {{WHO}}");
        let body = RequestBody::builder().raw(&raw).build();
        let tpl = SrTemplate::default();
        tpl.add_variable("WHO", "world");
        let resolve_body = body.resolve(&tpl).expect("Invalid resolve");
        let resolve_raw = resolve_body.raw().expect("Not a raw?");
        assert_eq!(resolve_raw.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(resolve_raw.payload(), "hello world");
    }

    #[test]
    fn test_resolve_body_with_file() {
        let request_body = RequestBodyFile::builder()
            .path("./assets/{{DOCUMENT_ID}}/report.xml")
            .content_type(Some("application/{{FORMAT}}+xml"))
            .build();
        let body = RequestBody::builder().file(&request_body).build();
        let tpl = SrTemplate::default();
        tpl.add_variable("DOCUMENT_ID", "1234");
        tpl.add_variable("FORMAT", "atom");

        let resolved_body = body.resolve(&tpl).expect("Invalid resolve");
        assert_eq!(resolved_body.body_type(), RequestBodyType::File);
        let rendered_body = resolved_body.file().expect("Is not file?");
        assert_eq!(rendered_body.path(), "./assets/1234/report.xml");
        assert!(
            rendered_body
                .content_type()
                .is_some_and(|t| t == "application/atom+xml"),
            "content-type: {:?}",
            rendered_body.content_type()
        );
    }
}
