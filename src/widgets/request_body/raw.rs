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

use glib::{object::ObjectExt, subclass::types::ObjectSubclassIsExt};

use crate::entities::{RawEncoding, RequestPayload};

use super::{BasePayloadPaneExt, PayloadType};

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use adw::subclass::bin::BinImpl;
    use glib::subclass::{InitializingObject, Signal};
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::{prelude::*, Revealer};
    use sourceview5::Buffer;
    use sourceview5::{prelude::*, LanguageManager};

    use crate::widgets::{BasePayloadPane, BasePayloadPaneImpl, CodeView, PayloadType, SearchBox};

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::RawPayloadPane)]
    #[template(resource = "/es/danirod/Cartero/raw_payload_pane.ui")]
    pub struct RawPayloadPane {
        #[template_child]
        view: TemplateChild<CodeView>,

        #[template_child]
        buffer: TemplateChild<Buffer>,

        #[template_child]
        search: TemplateChild<SearchBox>,

        #[template_child]
        search_revealer: TemplateChild<Revealer>,

        #[property(get = Self::format, set = Self::set_format, builder(PayloadType::default()))]
        _format: RefCell<PayloadType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RawPayloadPane {
        const NAME: &'static str = "CarteroRawPayloadPane";

        type Type = super::RawPayloadPane;
        type ParentType = BasePayloadPane;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for RawPayloadPane {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("changed").build()])
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.buffer
                .connect_changed(glib::clone!(@weak self as pane => move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }));
        }
    }

    impl WidgetImpl for RawPayloadPane {}

    impl BinImpl for RawPayloadPane {}

    impl BasePayloadPaneImpl for RawPayloadPane {}

    #[gtk::template_callbacks]
    impl RawPayloadPane {
        pub(super) fn payload(&self) -> Vec<u8> {
            let (start, end) = self.buffer.bounds();
            let text = self.buffer.text(&start, &end, true);
            Vec::from(text)
        }

        pub(super) fn set_payload(&self, payload: &[u8]) {
            let body = String::from_utf8_lossy(payload);
            self.buffer.set_text(&body);
        }

        fn format(&self) -> PayloadType {
            if let Some(language) = self.buffer.language() {
                if language.name() == "JSON" {
                    PayloadType::Json
                } else if language.name() == "XML" {
                    PayloadType::Xml
                } else {
                    PayloadType::Raw
                }
            } else {
                PayloadType::Raw
            }
        }

        fn set_format(&self, format: PayloadType) {
            let manager = LanguageManager::default();
            let language = match format {
                PayloadType::Json => manager.language("json"),
                PayloadType::Xml => manager.language("xml"),
                _ => None,
            };
            match language {
                Some(lang) => self.buffer.set_language(Some(&lang)),
                None => self.buffer.set_language(None),
            }
        }

        fn get_selected_text(&self) -> Option<String> {
            if self.buffer.has_selection() {
                if let Some((start, end)) = self.buffer.selection_bounds() {
                    let text = self.buffer.slice(&start, &end, false);
                    return Some(text.into());
                }
            }
            None
        }

        #[template_callback]
        fn on_search_requested(&self) {
            if !self.search_revealer.reveals_child() {
                self.search_revealer.set_visible(true);
                self.search_revealer.set_reveal_child(true);
            }
            let text = self.get_selected_text();
            self.search.init_search(text.as_deref());
            self.search.focus();
        }

        #[template_callback]
        fn on_search_close(&self) {
            self.search_revealer.set_reveal_child(false);
            self.search_revealer.set_visible(false);
            self.view.grab_focus();
        }
    }
}

glib::wrapper! {
    pub struct RawPayloadPane(ObjectSubclass<imp::RawPayloadPane>)
        @extends gtk::Widget, adw::Bin, super::BasePayloadPane,
        @implements gtk::Accessible, gtk::Buildable;
}

impl RawPayloadPane {
    pub fn connect_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "changed",
            true,
            glib::closure_local!(|ref pane| {
                f(pane);
            }),
        )
    }
}

impl BasePayloadPaneExt for RawPayloadPane {
    fn payload(&self) -> RequestPayload {
        let imp = self.imp();
        let content = imp.payload();
        let encoding = match self.format() {
            super::PayloadType::Json => RawEncoding::Json,
            super::PayloadType::Xml => RawEncoding::Xml,
            _ => RawEncoding::OctetStream,
        };
        RequestPayload::Raw { encoding, content }
    }

    fn set_payload(&self, payload: &RequestPayload) {
        if let RequestPayload::Raw { encoding, content } = payload {
            let imp = self.imp();
            imp.set_payload(content);
            let format = match encoding {
                RawEncoding::Json => PayloadType::Json,
                RawEncoding::Xml => PayloadType::Xml,
                _ => PayloadType::Raw,
            };
            self.set_format(format);
        }
    }
}
