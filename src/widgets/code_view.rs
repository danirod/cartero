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

use glib::{object::ObjectExt, Object};
use gtk::glib;

mod imp {
    use std::sync::OnceLock;

    use glib::object::Cast;
    use glib::subclass::Signal;
    use glib::value::ToValue;
    use gtk::gdk;
    use gtk::gio::SettingsBindFlags;
    use gtk::prelude::{SettingsExtManual, TextViewExt};
    use gtk::subclass::prelude::*;
    use gtk::{glib, WrapMode};
    use sourceview5::prelude::BufferExt;
    use sourceview5::subclass::view::ViewImpl;
    use sourceview5::StyleSchemeManager;

    use crate::app::CarteroApplication;

    #[derive(Default)]
    pub struct CodeView {}

    #[glib::object_subclass]
    impl ObjectSubclass for CodeView {
        const NAME: &'static str = "CarteroCodeView";
        type Type = super::CodeView;
        type ParentType = sourceview5::View;

        fn class_init(klass: &mut Self::Class) {
            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                "codeview.search",
            );
            klass.install_action("codeview.search", None, |widget, _, _| {
                widget.start_search();
            });
        }
    }

    impl ObjectImpl for CodeView {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("search-requested").build()])
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
            self.init_source_view_style();
        }
    }

    impl WidgetImpl for CodeView {}

    impl TextViewImpl for CodeView {}

    impl ViewImpl for CodeView {}

    impl CodeView {
        fn init_settings(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();
            let obj = self.obj();

            settings
                .bind("body-wrap", &*obj, "wrap-mode")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let enabled = variant.get::<bool>().expect("The variant is not a boolean");
                    let mode = match enabled {
                        true => WrapMode::WordChar,
                        false => WrapMode::None,
                    };
                    Some(mode.to_value())
                })
                .build();
            settings
                .bind("show-line-numbers", &*obj, "show-line-numbers")
                .flags(SettingsBindFlags::GET)
                .build();
            settings
                .bind("auto-indent", &*obj, "auto-indent")
                .flags(SettingsBindFlags::GET)
                .build();
            settings
                .bind("indent-style", &*obj, "insert-spaces-instead-of-tabs")
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
                .bind("tab-width", &*obj, "tab-width")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let width = variant.get::<String>().unwrap_or("4".into());
                    let value = width.parse::<i32>().unwrap_or(4);
                    Some(value.to_value())
                })
                .build();
            settings
                .bind("tab-width", &*obj, "indent-width")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let width = variant.get::<String>().unwrap_or("4".into());
                    let value = width.parse::<i32>().unwrap_or(4);
                    Some(value.to_value())
                })
                .build();
        }

        fn update_source_view_style(&self) {
            let obj = self.obj();
            let dark_mode = adw::StyleManager::default().is_dark();
            let color_theme = if dark_mode { "Adwaita-dark" } else { "Adwaita" };
            let theme = StyleSchemeManager::default().scheme(color_theme);
            let buffer = obj.buffer().downcast::<sourceview5::Buffer>().unwrap();
            match theme {
                Some(theme) => {
                    buffer.set_style_scheme(Some(&theme));
                    buffer.set_highlight_syntax(true);
                }
                None => {
                    buffer.set_highlight_syntax(false);
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
            let obj = self.obj();
            obj.connect_buffer_notify(glib::clone!(@weak self as panel => move |_| {
                panel.update_source_view_style();
            }));
        }
    }
}

glib::wrapper! {
    pub struct CodeView(ObjectSubclass<imp::CodeView>)
        @extends gtk::Widget, gtk::TextView, sourceview5::View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

impl CodeView {
    pub fn start_search(&self) {
        self.emit_by_name::<()>("search-requested", &[]);
    }
}

impl Default for CodeView {
    fn default() -> Self {
        Object::builder().build()
    }
}
