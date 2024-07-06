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

mod imp {
    use std::cell::RefCell;

    use adw::subclass::bin::BinImpl;
    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{gio::SettingsBindFlags, CompositeTemplate};
    use gtk::{prelude::*, WrapMode};
    use sourceview5::{prelude::*, LanguageManager};
    use sourceview5::{Buffer, StyleSchemeManager, View};

    use crate::app::CarteroApplication;
    use crate::widgets::{BasePayloadPane, BasePayloadPaneImpl, PayloadType};

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::RawPayloadPane)]
    #[template(resource = "/es/danirod/Cartero/raw_payload_pane.ui")]
    pub struct RawPayloadPane {
        #[template_child]
        view: TemplateChild<View>,

        #[template_child]
        buffer: TemplateChild<Buffer>,

        #[property(get = Self::payload, set = Self::set_payload, nullable, type = Option<glib::Bytes>)]
        _payload: RefCell<Option<glib::Bytes>>,

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
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for RawPayloadPane {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
            self.init_source_view_style();
        }
    }

    impl WidgetImpl for RawPayloadPane {}

    impl BinImpl for RawPayloadPane {}

    impl BasePayloadPaneImpl for RawPayloadPane {}

    impl RawPayloadPane {
        fn payload(&self) -> Option<glib::Bytes> {
            let (start, end) = self.buffer.bounds();
            let text = self.buffer.text(&start, &end, true);
            let bytes = glib::Bytes::from_owned(text);
            Some(bytes)
        }

        fn set_payload(&self, payload: Option<&glib::Bytes>) {
            match payload {
                Some(payload) => {
                    let body = String::from_utf8_lossy(payload);
                    self.buffer.set_text(&body);
                }
                None => {
                    self.buffer.set_text("");
                }
            }
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
    pub struct RawPayloadPane(ObjectSubclass<imp::RawPayloadPane>)
        @extends gtk::Widget, adw::Bin, super::BasePayloadPane,
        @implements gtk::Accessible, gtk::Buildable;
}
