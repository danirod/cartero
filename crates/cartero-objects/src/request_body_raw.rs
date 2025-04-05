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
    pub struct RequestBodyRaw(ObjectSubclass<imp::RequestBodyRaw>) @extends crate::RequestBodyData;
}

impl Default for RequestBodyRaw {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl RequestBodyRaw {
    pub fn new(raw_type: RequestBodyRawType, initial: &[u8]) -> Self {
        let bytes = glib::Bytes::from(initial);
        Object::builder()
            .property("payload-type", raw_type)
            .property("payload", bytes)
            .build()
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestBodyRawType")]
pub enum RequestBodyRawType {
    #[default]
    #[enum_value(name = "OCTET_STREAM", nick = "Octet Stream")]
    OctetStream,
    #[enum_value(name = "JSON", nick = "JSON")]
    Json,
    #[enum_value(name = "XML", nick = "XML")]
    Xml,
}

mod imp {
    use std::cell::RefCell;

    use crate::RequestBodyDataImpl;

    use super::*;
    use glib::Properties;

    use super::RequestBodyRawType;

    #[derive(Properties)]
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
        payload: RefCell<glib::Bytes>,
    }

    impl Default for RequestBodyRaw {
        fn default() -> Self {
            Self {
                payload_type: Default::default(),
                payload: glib::Bytes::from_static(&[]).into(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyRaw {
        const NAME: &'static str = "CarteroRequestBodyRaw";
        type Type = super::RequestBodyRaw;
        type ParentType = crate::RequestBodyData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBodyRaw {}

    impl RequestBodyDataImpl for RequestBodyRaw {
        fn body_type(&self) -> crate::RequestBodyType {
            crate::RequestBodyType::Raw
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::test::assert_emits_signal, RequestBodyDataExt, RequestBodyRawType, RequestBodyType,
    };

    use super::RequestBodyRaw;

    #[test]
    fn test_defaults() {
        let raw = RequestBodyRaw::default();
        assert_eq!(raw.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(raw.payload().len(), 0);
    }

    #[test]
    fn test_new() {
        let raw = RequestBodyRaw::new(RequestBodyRawType::Xml, "<?xml?>".as_bytes());
        assert_eq!(raw.payload_type(), RequestBodyRawType::Xml);
        assert_eq!(raw.payload().len(), 7);
        let payload = raw.payload();
        let contents = String::from_utf8_lossy(payload.as_ref());
        assert_eq!(contents, "<?xml?>");
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
            let new_body = "hello world".as_bytes();
            raw.set_payload(glib::Bytes::from(new_body));
        });
        assert_eq!(11, raw.payload().len());
    }

    #[test]
    pub fn test_body_type() {
        let body: RequestBodyRaw = RequestBodyRaw::default();
        assert_eq!(body.body_type(), RequestBodyType::Raw);
    }
}
