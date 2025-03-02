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

use crate::entities::{EndpointData, RequestExportType};

use super::{BaseExportPaneExt, CodeExportService};

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use adw::subclass::bin::BinImpl;
    use gettextrs::gettext;
    use glib::subclass::{InitializingObject, Signal};
    use glib::Properties;
    use gtk::gdk::Display;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, template_callbacks, Button};
    use gtk::{CompositeTemplate, Revealer};
    use sourceview5::Buffer;
    use sourceview5::{prelude::*, LanguageManager};

    use crate::app::CarteroApplication;
    use crate::widgets::{BaseExportPane, BaseExportPaneImpl, CodeView, ExportType, SearchBox};
    use crate::win::CarteroWindow;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::CodeExportPane)]
    #[template(resource = "/es/danirod/Cartero/code_export_pane.ui")]
    pub struct CodeExportPane {
        #[template_child]
        view: TemplateChild<CodeView>,

        #[template_child]
        buffer: TemplateChild<Buffer>,

        #[template_child]
        copy_button: TemplateChild<Button>,

        #[template_child]
        search: TemplateChild<SearchBox>,

        #[template_child]
        search_revealer: TemplateChild<Revealer>,

        #[property(get = Self::format, set = Self::set_format, builder(ExportType::default()))]
        _format: RefCell<ExportType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CodeExportPane {
        const NAME: &'static str = "CarteroCodeExportPane";

        type Type = super::CodeExportPane;
        type ParentType = BaseExportPane;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CodeExportPane {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("changed").build()])
        }
    }

    impl WidgetImpl for CodeExportPane {}

    impl BinImpl for CodeExportPane {}

    impl BaseExportPaneImpl for CodeExportPane {}

    #[template_callbacks]
    impl CodeExportPane {
        #[template_callback]
        fn on_copy_button_clicked(&self) {
            if let Some(display) = Display::default() {
                let clipboard = display.clipboard();
                let buf_contents = self.buffer_content();
                let contents = String::from_utf8_lossy(buf_contents.as_ref());
                clipboard.set_text(contents.as_ref());

                let app = CarteroApplication::get();

                let window = match app.active_window() {
                    Some(window) => window.downcast::<CarteroWindow>().unwrap(),
                    None => CarteroWindow::new(&app),
                };

                window.toast_message(&gettext("Content copied to the clipboard"));
            }
        }

        #[allow(unused)]
        pub(super) fn buffer_content(&self) -> Vec<u8> {
            let (start, end) = self.buffer.bounds();
            let text = self.buffer.text(&start, &end, true);
            Vec::from(text)
        }

        #[allow(unused)]
        pub(super) fn set_buffer_content(&self, payload: &[u8]) {
            let body = String::from_utf8_lossy(payload);
            self.buffer.set_text(&body);
        }

        fn format(&self) -> ExportType {
            // Maybe in a future we're gonna add axios and others here, not sure
            // if that thing should be in another widget or this could could be
            // reutilized for that purpose, since we're just gonna use anyways a
            // textbox, [axios export issue](https://github.com/danirod/cartero/issues/64)
            ExportType::Curl
        }

        fn set_format(&self, format: ExportType) {
            let manager = LanguageManager::default();

            // TODO: I'm not really sure what the language id should be here,
            // already tried bash shellscript sh shell etc.
            let language = match format {
                ExportType::Curl => manager.language("shellscript"),
                _ => None,
            };

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
}

glib::wrapper! {
    pub struct CodeExportPane(ObjectSubclass<imp::CodeExportPane>)
        @extends gtk::Widget, adw::Bin, super::BaseExportPane,
        @implements gtk::Accessible, gtk::Buildable;
}

impl CodeExportPane {
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

impl BaseExportPaneExt for CodeExportPane {
    fn request_export_type(&self) -> RequestExportType {
        // TODO: Maybe we could extract url and others from the service which is
        // gonna generate curl output or idk, like reparse the generated content
        // to extract its data and regenerate a new endpoint data.
        match self.format() {
            super::ExportType::Curl => RequestExportType::Curl(EndpointData::default()),
            _ => RequestExportType::None,
        }
    }

    fn set_request_export_type(&self, req_export_type: &RequestExportType) {
        if let RequestExportType::Curl(data) = req_export_type {
            let service = CodeExportService::new(data.clone());
            let imp = self.imp();

            if let Ok(command) = service.generate() {
                imp.set_buffer_content(command.as_bytes());
            }
        }
    }
}
