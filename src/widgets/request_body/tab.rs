// Copyright 2024 the Cartero authors
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

use std::sync::OnceLock;

use glib::object::CastNone;
use gtk::subclass::prelude::*;

use crate::entities::{RawEncoding, RequestPayload};

use super::{BasePayloadPaneExt, FormdataPayloadPane, RawPayloadPane, UrlencodedPayloadPane};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroPayloadType")]
pub enum PayloadType {
    #[default]
    None,
    UrlEncoded,
    MultipartFormData,
    Json,
    Xml,
    Raw,
}

impl PayloadType {
    pub fn types() -> &'static [PayloadType] {
        static TYPES: OnceLock<Vec<PayloadType>> = OnceLock::new();
        TYPES.get_or_init(|| {
            vec![
                PayloadType::None,
                PayloadType::UrlEncoded,
                PayloadType::MultipartFormData,
                PayloadType::Json,
                PayloadType::Xml,
                PayloadType::Raw,
            ]
        })
    }
}

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use adw::ComboRow;
    use glib::{subclass::InitializingObject, Properties};
    use gtk::template_callbacks;
    use gtk::Separator;
    use gtk::{CompositeTemplate, Stack};

    use crate::widgets::BasePayloadPane;
    use crate::widgets::FormdataPayloadPane;
    use crate::widgets::RawPayloadPane;
    use crate::widgets::UrlencodedPayloadPane;

    use super::PayloadType;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::PayloadTab)]
    #[template(resource = "/es/danirod/Cartero/payload_tab.ui")]
    pub struct PayloadTab {
        #[template_child]
        stack: TemplateChild<Stack>,

        #[template_child]
        combo: TemplateChild<ComboRow>,

        #[template_child]
        sep: TemplateChild<Separator>,

        #[template_child]
        raw: TemplateChild<RawPayloadPane>,

        #[template_child]
        urlencoded: TemplateChild<UrlencodedPayloadPane>,

        #[template_child]
        formdata: TemplateChild<FormdataPayloadPane>,

        #[property(get = Self::payload_type, set = Self::set_payload_type, builder(PayloadType::default()))]
        _payload_type: RefCell<PayloadType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PayloadTab {
        const NAME: &'static str = "CarteroPayloadTab";
        type Type = super::PayloadTab;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PayloadTab {
        fn constructed(&self) {
            self.parent_constructed();
            self.on_selection_changed();
        }
    }

    impl WidgetImpl for PayloadTab {}

    impl BoxImpl for PayloadTab {}

    #[template_callbacks]
    impl PayloadTab {
        #[template_callback]
        fn on_selection_changed(&self) {
            let payload_type = self.payload_type();
            let tab = match payload_type {
                PayloadType::None => "none",
                PayloadType::Raw => "raw",
                PayloadType::Json => "raw",
                PayloadType::Xml => "raw",
                PayloadType::UrlEncoded => "urlencoded",
                PayloadType::MultipartFormData => "formdata",
            };
            self.stack.set_visible_child_name(tab);

            self.sep.set_visible(payload_type != PayloadType::None);
            self.raw.set_format(payload_type);
        }

        fn payload_type(&self) -> PayloadType {
            let n_item = self.combo.selected();
            PayloadType::types()[n_item as usize]
        }

        fn set_payload_type(&self, pt: PayloadType) {
            let pos = PayloadType::types().iter().position(|&t| t == pt).unwrap();
            self.combo.set_selected(pos as u32);
        }

        pub(super) fn get_active_widget(&self) -> Option<BasePayloadPane> {
            match self.payload_type() {
                PayloadType::None => None,
                PayloadType::UrlEncoded => {
                    Some(self.urlencoded.upcast_ref::<BasePayloadPane>().clone())
                }
                PayloadType::MultipartFormData => {
                    Some(self.formdata.upcast_ref::<BasePayloadPane>().clone())
                }
                PayloadType::Json | PayloadType::Xml | PayloadType::Raw => {
                    Some(self.raw.upcast_ref::<BasePayloadPane>().clone())
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct PayloadTab(ObjectSubclass<imp::PayloadTab>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable;
}

impl PayloadTab {
    pub fn set_payload(&self, payload: &RequestPayload) {
        let payload_type = match payload {
            RequestPayload::None => PayloadType::None,
            RequestPayload::Urlencoded(_) => PayloadType::UrlEncoded,
            RequestPayload::Multipart { params: _ } => PayloadType::MultipartFormData,
            RequestPayload::Raw {
                encoding,
                content: _,
            } => match encoding {
                RawEncoding::Json => PayloadType::Json,
                RawEncoding::Xml => PayloadType::Xml,
                RawEncoding::OctetStream => PayloadType::Raw,
            },
        };
        self.set_payload_type(payload_type);

        let widget = self.imp().get_active_widget();
        match self.payload_type() {
            PayloadType::None => {}
            PayloadType::UrlEncoded => {
                let widget = widget.and_downcast::<UrlencodedPayloadPane>().unwrap();
                widget.set_payload(payload);
            }
            PayloadType::MultipartFormData => {
                let widget = widget.and_downcast::<FormdataPayloadPane>().unwrap();
                widget.set_payload(payload);
            }
            PayloadType::Json | PayloadType::Xml | PayloadType::Raw => {
                let widget = widget.and_downcast::<RawPayloadPane>().unwrap();
                widget.set_payload(payload);
            }
        }
    }

    pub fn payload(&self) -> RequestPayload {
        let widget = self.imp().get_active_widget();
        match self.payload_type() {
            PayloadType::None => RequestPayload::None,
            PayloadType::UrlEncoded => {
                let widget = widget.and_downcast::<UrlencodedPayloadPane>().unwrap();
                widget.payload()
            }
            PayloadType::MultipartFormData => {
                let widget = widget.and_downcast::<FormdataPayloadPane>().unwrap();
                widget.payload()
            }
            PayloadType::Json | PayloadType::Xml | PayloadType::Raw => {
                let widget = widget.and_downcast::<RawPayloadPane>().unwrap();
                widget.payload()
            }
        }
    }
}
