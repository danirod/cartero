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

use crate::{RequestBodyData, RequestBodyMultipart, RequestBodyRaw, RequestBodyUrlencoded};

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
    /// let raw = RequestBodyRaw::new(RequestBodyRawType::Json, data.as_bytes());
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
}

mod builder {
    use glib::object::ObjectBuilder;

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
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use crate::{RequestBodyData, RequestBodyDataExt};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestBody)]
    pub struct RequestBody {
        #[property(get, set = Self::set_body_type, name = "body-type", builder(RequestBodyType::None))]
        body_type: RefCell<RequestBodyType>,

        #[property(get, set = Self::set_body_data, explicit_notify, name = "body-data", nullable)]
        body_data: RefCell<Option<RequestBodyData>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBody {
        const NAME: &'static str = "CarteroRequestBody";
        type Type = super::RequestBody;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBody {}

    impl RequestBody {
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
                glib::g_critical!("Cartero", "set_body_data() was called with a RequestBodyData of invalid RequestBodyType for this RequestBody object");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use gio::prelude::ListModelExt;
    use glib::object::CastNone;

    use crate::{
        utils::test::{assert_emits_signal, assert_emits_signals, assert_not_emits_signal},
        Field, FieldTable, RequestBodyData, RequestBodyDataExt, RequestBodyMultipart,
        RequestBodyRaw, RequestBodyRawType, RequestBodyType, RequestBodyUrlencoded,
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
        let payload = RequestBodyRaw::new(RequestBodyRawType::Json, "[1, 2, 4]".as_bytes());
        let body = RequestBody::new(RequestBodyType::Raw, Some(payload));
        assert_eq!(RequestBodyType::Raw, body.body_type());
        let body_data = body.body_data().and_downcast::<RequestBodyRaw>().unwrap();
        assert_eq!(body_data.payload_type(), RequestBodyRawType::Json);
        assert_eq!(body_data.payload().len(), 9);
    }

    #[test]
    pub fn set_body_type_changes_data_type() {
        let body = RequestBody::new(RequestBodyType::UrlEncoded, RequestBodyData::NONE);
        assert!(body
            .body_data()
            .is_some_and(|data| data.body_type() == RequestBodyType::UrlEncoded));
        assert_emits_signals(&body, &["notify::body-type", "notify::body-data"], || {
            body.set_body_type(RequestBodyType::Multipart)
        });
        assert!(body
            .body_data()
            .is_some_and(|data| data.body_type() == RequestBodyType::Multipart));
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
            .payload(&glib::Bytes::from(b"hello world"))
            .build();
        let body = RequestBody::builder().raw(&raw).build();
        assert_eq!(body.body_type(), RequestBodyType::Raw);
        let data = body.raw().unwrap();
        assert_eq!(data.payload_type(), RequestBodyRawType::OctetStream);
        let bytes = data.payload().into_data();
        assert_eq!(bytes.as_ref(), b"hello world");
    }
}
