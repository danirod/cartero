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

use std::path::PathBuf;

use glib::Object;
use gtk::gio::{ListModel, ListStore};
use gtk::glib;
use gtk::prelude::TextViewExt;
use gtk::prelude::*;
use sourceview5::prelude::BufferExt;
use sourceview5::LanguageManager;

use crate::entities::ResponseData;
use crate::objects::KeyValueItem;
use glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::bin::BinImpl;
    use glib::object::Cast;
    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::gio::SettingsBindFlags;
    use gtk::subclass::prelude::*;
    use gtk::{
        subclass::widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
        Box, CompositeTemplate, Label, TemplateChild,
    };
    use gtk::{Spinner, Stack, WrapMode};
    use sourceview5::prelude::BufferExt;
    use sourceview5::StyleSchemeManager;

    use crate::app::CarteroApplication;
    use crate::widgets::ResponseHeaders;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::ResponsePanel)]
    #[template(resource = "/es/danirod/Cartero/response_panel.ui")]
    pub struct ResponsePanel {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub response_headers: TemplateChild<ResponseHeaders>,
        #[template_child]
        pub response_body: TemplateChild<sourceview5::View>,
        #[template_child]
        pub response_meta: TemplateChild<Box>,
        #[template_child]
        pub status_code: TemplateChild<Label>,
        #[template_child]
        pub duration: TemplateChild<Label>,
        #[template_child]
        pub response_size: TemplateChild<Label>,
        #[template_child]
        pub spinner: TemplateChild<Spinner>,
        #[template_child]
        pub metadata_stack: TemplateChild<Stack>,

        #[property(get = Self::spinning, set = Self::set_spinning)]
        _spinning: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResponsePanel {
        const NAME: &'static str = "CarteroResponsePanel";
        type Type = super::ResponsePanel;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ResponsePanel {
        fn constructed(&self) {
            self.parent_constructed();

            self.init_settings();
            self.init_source_view_style();
        }
    }

    impl WidgetImpl for ResponsePanel {}

    impl BinImpl for ResponsePanel {}

    impl ResponsePanel {
        fn init_settings(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();

            settings
                .bind("body-wrap", &*self.response_body, "wrap-mode")
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
                .bind(
                    "show-line-numbers",
                    &*self.response_body,
                    "show-line-numbers",
                )
                .flags(SettingsBindFlags::GET)
                .build();
        }

        fn update_source_view_style(&self) {
            let dark_mode = adw::StyleManager::default().is_dark();
            let color_theme = if dark_mode { "Adwaita-dark" } else { "Adwaita" };
            let theme = StyleSchemeManager::default().scheme(color_theme);

            let buffer = self
                .response_body
                .buffer()
                .downcast::<sourceview5::Buffer>()
                .unwrap();
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
        }

        fn spinning(&self) -> bool {
            self.metadata_stack
                .visible_child()
                .is_some_and(|w| w.is::<Spinner>())
        }

        fn set_spinning(&self, spinning: bool) {
            self.stack.set_visible_child_name("response");
            let widget: &gtk::Widget = if spinning {
                self.spinner.upcast_ref()
            } else {
                self.response_meta.upcast_ref()
            };
            self.metadata_stack.set_visible_child(widget);
        }
    }
}

glib::wrapper! {
    pub struct ResponsePanel(ObjectSubclass<imp::ResponsePanel>)
        @extends gtk::Widget, gtk::Overlay,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ResponsePanel {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Whether to use SI units or base 2 units?
fn format_bytes(count: usize) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut total = count as f64;
    let mut unit = 0;

    while total > 1024.0 {
        total /= 1024.0;
        unit += 1;
    }

    if unit > 0 {
        format!("{:.3} {}", total, units[unit])
    } else {
        format!("{} {}", total, units[unit])
    }
}

impl ResponsePanel {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn start_request(&self) {
        let imp = self.imp();

        imp.metadata_stack.set_visible_child(&*imp.spinner);
    }

    pub fn assign_from_response(&self, resp: &ResponseData) {
        let imp = self.imp();

        let mut headers = resp.headers.clone();
        headers.sort();
        let headers: Vec<KeyValueItem> = headers
            .iter()
            .map(|kv| KeyValueItem::from(kv.clone()))
            .collect();

        let store = ListStore::with_type(KeyValueItem::static_type());
        store.extend_from_slice(&headers);
        let model = store.upcast::<ListModel>();
        imp.response_headers.set_headers(Some(&model));

        let status = format!("HTTP {}", resp.status_code);
        imp.status_code.set_text(&status);
        imp.status_code.set_visible(true);

        let duration = format!("{} s", resp.seconds());
        imp.duration.set_text(&duration);
        imp.duration.set_visible(true);

        let size = format_bytes(resp.size);
        imp.response_size.set_text(&size);
        imp.response_size.set_visible(true);

        imp.metadata_stack.set_visible_child(&*imp.response_meta);

        let buffer = imp
            .response_body
            .buffer()
            .downcast::<sourceview5::Buffer>()
            .unwrap();

        buffer.set_text(&resp.body_str());

        let language = resp
            .headers
            .header("Content-Type")
            .map(|ctypes| ctypes[0])
            .and_then(|ctype| {
                let ctype = match ctype.split_once(';') {
                    Some((c, _)) => c,
                    None => ctype,
                };
                LanguageManager::default().guess_language(Option::<PathBuf>::None, Some(ctype))
            });

        match language {
            Some(language) => buffer.set_language(Some(&language)),
            None => buffer.set_language(None),
        };
    }
}
