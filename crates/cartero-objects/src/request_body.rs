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

impl RequestBody {
    pub fn new<T>(body_type: RequestBodyType, body_data: &T) -> Self
    where
        T: IsA<RequestBodyData>,
    {
        let body = Self::default();
        body.set_body_type(body_type);
        body.set_body_data(Some(body_data.clone()));
        body
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestBodyType")]
pub enum RequestBodyType {
    #[enum_value(name = "URL_ENCODED", nick = "URL-Encoded")]
    UrlEncoded,
    #[enum_value(name = "MULTIPART", nick = "Multipart")]
    Multipart,
    #[default]
    #[enum_value(name = "RAW", nick = "Raw")]
    Raw,
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use crate::{RequestBodyData, RequestBodyMultipart, RequestBodyRaw, RequestBodyUrlencoded};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestBody)]
    pub struct RequestBody {
        #[property(get, set, name = "body-type", builder(RequestBodyType::default()))]
        body_type: RefCell<RequestBodyType>,

        #[property(get, set, name = "body-data", nullable)]
        body_data: RefCell<Option<RequestBodyData>>,
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

            self.obj().connect_body_type_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |body| {
                    let next = match body.body_type() {
                        RequestBodyType::UrlEncoded => {
                            Some(RequestBodyUrlencoded::default().upcast::<RequestBodyData>())
                        }
                        RequestBodyType::Multipart => {
                            Some(RequestBodyMultipart::default().upcast::<RequestBodyData>())
                        }
                        RequestBodyType::Raw => {
                            Some(RequestBodyRaw::default().upcast::<RequestBodyData>())
                        }
                    };
                    imp.body_data.replace(next);
                    body.notify_body_data();
                }
            ));
        }
    }
}
