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

use adw::prelude::*;
use adw::subclass::prelude::*;

mod imp {
    use std::cell::RefCell;

    use crate::widgets::{CodeView, SearchBox};

    use super::*;
    use cartero_objects::{RequestBodyRaw, RequestBodyRawType};
    use glib::{subclass::InitializingObject, Properties};
    use gtk::{CompositeTemplate, Revealer};
    use sourceview5::{prelude::BufferExt, Buffer, LanguageManager};

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Raw)]
    #[template(resource = "/es/danirod/Cartero/raw_body_pane.ui")]
    pub struct Raw {
        #[property(get, set)]
        payload: RefCell<RequestBodyRaw>,

        #[template_child]
        buffer: TemplateChild<Buffer>,
        #[template_child]
        view: TemplateChild<CodeView>,
        #[template_child]
        search_revealer: TemplateChild<Revealer>,
        #[template_child]
        search: TemplateChild<SearchBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Raw {
        const NAME: &'static str = "CarteroRawBodyPane";
        type Type = super::Raw;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Raw {
        fn constructed(&self) {
            self.parent_constructed();

            self.reload_payload();
            self.obj().connect_payload_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_: &super::Raw| {
                    imp.reload_payload();
                }
            ));
        }
    }

    impl WidgetImpl for Raw {}

    impl BinImpl for Raw {}

    #[gtk::template_callbacks]
    impl Raw {
        fn reload_payload(&self) {
            self.obj()
                .payload()
                .bind_property("payload", &*self.buffer, "text")
                .sync_create()
                .bidirectional()
                .build();
            let content_type = self.payload.borrow().payload_type();
            let language = get_sourceview_language(content_type);
            self.buffer.set_language(language.as_ref());
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

    fn get_sourceview_language(format: RequestBodyRawType) -> Option<sourceview5::Language> {
        let manager = LanguageManager::default();
        match format {
            RequestBodyRawType::Json => manager.language("json"),
            RequestBodyRawType::Xml => manager.language("xml"),
            _ => None,
        }
    }
}

glib::wrapper! {
    pub struct Raw(ObjectSubclass<imp::Raw>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
