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
use serde_json::Value;
use sourceview5::prelude::BufferExt;
use sourceview5::LanguageManager;

use crate::entities::ResponseData;
use crate::objects::KeyValueItem;
use glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use std::cell::RefCell;

    use crate::widgets::{CodeView, ResponseHeaders, SearchBox};
    use adw::prelude::*;
    use adw::subclass::bin::BinImpl;
    use glib::object::Cast;
    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{
        subclass::widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
        Box, CompositeTemplate, Label, TemplateChild,
    };
    use gtk::{Revealer, Spinner, Stack};

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::ResponsePanel)]
    #[template(resource = "/es/danirod/Cartero/response_panel.ui")]
    pub struct ResponsePanel {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub response_headers: TemplateChild<ResponseHeaders>,
        #[template_child]
        pub response_body: TemplateChild<CodeView>,
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
        #[template_child]
        buffer: TemplateChild<sourceview5::Buffer>,
        #[template_child]
        search: TemplateChild<SearchBox>,
        #[template_child]
        search_revealer: TemplateChild<Revealer>,

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
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ResponsePanel {}

    impl WidgetImpl for ResponsePanel {}

    impl BinImpl for ResponsePanel {}

    #[gtk::template_callbacks]
    impl ResponsePanel {
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
            self.response_body.grab_focus();
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

        let status = format!("• HTTP {}", resp.status_code);
        imp.status_code.set_text(&status);
        imp.status_code.set_visible(true);
        let status_color = match resp.status_code {
            200..=299 => "success",
            400..=499 => "warning",
            500..=599 => "error",
            _ => "neutral",
        };

        // This will stop classes with higher priorities to override the currently
        // needed one by resetting the classes, see [this](https://github.com/danirod/cartero/issues/83).
        let possible_classes = vec!["success", "warning", "error", "neutral"];
        for css_class in possible_classes {
            imp.status_code.remove_css_class(css_class);
        }

        imp.status_code.add_css_class(status_color);

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

        if resp.is_json() {
            let json = serde_json::from_str(&resp.body_str())
                .and_then(|text: Value| serde_json::to_string_pretty(&text));
            if let Ok(json) = json {
                buffer.set_text(&json);
            }
        }

        let language = if resp.is_json() {
            LanguageManager::default().language("json")
        } else if resp.is_xml() {
            LanguageManager::default().language("xml")
        } else {
            resp.headers
                .header("Content-Type")
                .map(|ctypes| ctypes[0])
                .and_then(|ctype| {
                    let ctype = match ctype.split_once(';') {
                        Some((c, _)) => c,
                        None => ctype,
                    };
                    LanguageManager::default().guess_language(Option::<PathBuf>::None, Some(ctype))
                })
        };

        match language {
            Some(language) => buffer.set_language(Some(&language)),
            None => buffer.set_language(None),
        };
    }
}
