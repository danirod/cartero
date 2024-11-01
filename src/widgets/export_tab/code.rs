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
    use gtk::{gio::SettingsBindFlags, CompositeTemplate};
    use gtk::{prelude::*, template_callbacks, Button, WrapMode};
    use sourceview5::{prelude::*, LanguageManager};
    use sourceview5::{Buffer, StyleSchemeManager, View};

    use crate::app::CarteroApplication;
    use crate::widgets::{BaseExportPane, BaseExportPaneImpl, ExportType};
    use crate::win::CarteroWindow;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::CodeExportPane)]
    #[template(resource = "/es/danirod/Cartero/code_export_pane.ui")]
    pub struct CodeExportPane {
        #[template_child]
        view: TemplateChild<View>,

        #[template_child]
        buffer: TemplateChild<Buffer>,

        #[template_child]
        copy_button: TemplateChild<Button>,

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

        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
            self.init_source_view_style();
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

        fn init_settings(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();

            settings
                .bind("body-wrap", &*self.view, "wrap-mode")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let enabled = variant.get::<bool>().expect("The variant is not a boolean");
                    let mode = match enabled {
                        true => WrapMode::Word,
                        false => WrapMode::None,
                    };
                    Some(mode.to_value())
                })
                .build();

            settings
                .bind("show-line-numbers", &*self.view, "show-line-numbers")
                .flags(SettingsBindFlags::GET)
                .build();
            settings
                .bind("auto-indent", &*self.view, "auto-indent")
                .flags(SettingsBindFlags::GET)
                .build();
            settings
                .bind("indent-style", &*self.view, "insert-spaces-instead-of-tabs")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let mode = variant
                        .get::<String>()
                        .expect("The variant is not a string");
                    let use_spaces = mode == "spaces";
                    Some(use_spaces.to_value())
                })
                .build();
            settings
                .bind("tab-width", &*self.view, "tab-width")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let width = variant.get::<String>().unwrap_or("4".into());
                    let value = width.parse::<i32>().unwrap_or(4);
                    Some(value.to_value())
                })
                .build();
            settings
                .bind("tab-width", &*self.view, "indent-width")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let width = variant.get::<String>().unwrap_or("4".into());
                    let value = width.parse::<i32>().unwrap_or(4);
                    Some(value.to_value())
                })
                .build();
        }

        fn update_source_view_style(&self) {
            let dark_mode = adw::StyleManager::default().is_dark();
            let color_theme = if dark_mode { "Adwaita-dark" } else { "Adwaita" };
            let theme = StyleSchemeManager::default().scheme(color_theme);

            match theme {
                Some(theme) => {
                    self.buffer.set_style_scheme(Some(&theme));
                    self.buffer.set_highlight_syntax(true);
                }
                None => {
                    self.buffer.set_highlight_syntax(false);
                }
            }
        }

        fn init_source_view_style(&self) {
            self.update_source_view_style();
            adw::StyleManager::default().connect_dark_notify(
                glib::clone!(@weak self as panel => move |_| {
                    panel.update_source_view_style();
                }),
            );
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
