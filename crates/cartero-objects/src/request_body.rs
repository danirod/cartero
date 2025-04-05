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
    pub struct RequestBody(ObjectSubclass<imp::RequestBody>);
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

    pub fn urlencoded(&self) -> Option<RequestBodyUrlencoded> {
        if self.body_type() == RequestBodyType::UrlEncoded {
            self.body_data().and_downcast::<RequestBodyUrlencoded>()
        } else {
            None
        }
    }

    pub fn multipart(&self) -> Option<RequestBodyMultipart> {
        if self.body_type() == RequestBodyType::Multipart {
            self.body_data().and_downcast::<RequestBodyMultipart>()
        } else {
            None
        }
    }

    pub fn raw(&self) -> Option<RequestBodyRaw> {
        if self.body_type() == RequestBodyType::Raw {
            self.body_data().and_downcast::<RequestBodyRaw>()
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestBodyType")]
pub enum RequestBodyType {
    #[default]
    #[enum_value(name = "NONE", nick = "None")]
    None,
    #[enum_value(name = "URL_ENCODED", nick = "URL-Encoded")]
    UrlEncoded,
    #[enum_value(name = "MULTIPART", nick = "Multipart")]
    Multipart,
    #[enum_value(name = "RAW", nick = "Raw")]
    Raw,
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
}
