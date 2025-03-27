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

#[derive(Copy, Clone, Default, PartialEq, Eq, glib::Enum)]
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

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestBodyRaw)]
    pub struct RequestBodyRaw {
        #[property(get, set, name = "body-type", builder(RequestBodyRawType::OctetStream))]
        body_type: RefCell<RequestBodyRawType>,

        #[property(get, set, nullable)]
        payload: RefCell<Option<glib::Bytes>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyRaw {
        const NAME: &'static str = "CarteroRequestBodyRaw";
        type Type = super::RequestBodyRaw;
        type ParentType = crate::RequestBodyData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestBodyRaw {}

    impl RequestBodyDataImpl for RequestBodyRaw {}
}
