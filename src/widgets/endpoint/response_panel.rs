// Copyright 2024-2026 the Cartero authors
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

use cartero_http::RequestError;
use cartero_objects::Response;
use formatx::formatx;
use gettextrs::gettext;
use glib::Object;
use gtk::glib;
use gtk::prelude::TextViewExt;
use gtk::prelude::*;
use serde_json::Value;
use sourceview5::LanguageManager;
use sourceview5::prelude::BufferExt;

use glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use std::cell::RefCell;

    use crate::widgets::endpoint::ResponseHeaders;
    use crate::widgets::{CodeView, ErrorPane, SearchBox};
    use adw::subclass::bin::BinImpl;
    use adw::{ToastOverlay, prelude::*};
    use cartero_http::RequestError;
    use cartero_objects::Response;
    use gettextrs::gettext;
    use glib::Properties;
    use glib::object::Cast;
    use glib::subclass::InitializingObject;
    use gtk::gdk::{ContentProvider, Display};
    use gtk::gio::{SimpleAction, SimpleActionGroup};
    use gtk::subclass::prelude::*;
    use gtk::{
        Box, CompositeTemplate, Label, TemplateChild,
        subclass::widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
    };
    use gtk::{Revealer, Spinner, Stack};

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::ResponsePanel)]
    #[template(resource = "/es/danirod/Cartero/response_panel.ui")]
    pub struct ResponsePanel {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        error_page: TemplateChild<ErrorPane>,
        #[template_child]
        pub response_headers: TemplateChild<ResponseHeaders>,
        #[template_child]
        response_body_stack: TemplateChild<gtk::Stack>,
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
        pub response_url: TemplateChild<gtk::Entry>,
        #[template_child]
        search: TemplateChild<SearchBox>,
        #[template_child]
        search_revealer: TemplateChild<Revealer>,
        #[template_child]
        toaster: TemplateChild<ToastOverlay>,
        #[property(get, set = Self::set_response, nullable)]
        response: RefCell<Option<Response>>,
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
    impl ObjectImpl for ResponsePanel {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_actions();
        }
    }

    impl WidgetImpl for ResponsePanel {}

    impl BinImpl for ResponsePanel {}

    #[gtk::template_callbacks]
    impl ResponsePanel {
        fn init_actions(&self) {
            let obj = self.obj();

            let action_force_binary_render = SimpleAction::new("force-binary-render", None);
            action_force_binary_render.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.obj().render_response_body_as_text();
                    imp.response_body_stack.set_visible_child_name("text");
                }
            ));

            let action_copy_response_url = SimpleAction::new("copy-response-url", None);
            action_copy_response_url.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    let content = {
                        let blob = imp
                            .obj()
                            .response()
                            .map(|response| response.effective_url())
                            .unwrap_or_default();
                        let blob = glib::Bytes::from(blob.as_bytes());
                        ContentProvider::new_union(&[
                            ContentProvider::for_bytes("text/plain", &blob),
                            ContentProvider::for_bytes("text/plain;charset=utf-8", &blob),
                        ])
                    };
                    if let Some(display) = Display::default() {
                        let clipboard = display.clipboard();
                        clipboard.set_content(Some(&content)).unwrap();

                        let msg = gettext("Content copied to the clipboard");
                        let toast = adw::Toast::new(&msg);
                        imp.toaster.add_toast(toast);
                    }
                }
            ));

            let action_group = SimpleActionGroup::new();
            action_group.add_action(&action_force_binary_render);
            action_group.add_action(&action_copy_response_url);
            obj.insert_action_group("response", Some(&action_group));
        }

        fn set_response(&self, response: Option<Response>) {
            let initial_stack_page = if response.as_ref().is_some_and(|r| r.is_binary()) {
                "binary"
            } else {
                "text"
            };
            self.response_body_stack
                .set_visible_child_name(initial_stack_page);
            self.response.replace(response);
        }

        fn spinning(&self) -> bool {
            self.metadata_stack
                .visible_child()
                .is_some_and(|w| w.is::<Spinner>())
        }

        fn set_spinning(&self, spinning: bool) {
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

        pub(super) fn present_error(&self, error: RequestError) {
            self.error_page.set_error(error);
            self.stack.set_visible_child_name("error");
        }

        pub(super) fn show_response(&self) {
            self.stack.set_visible_child_name("response");
        }
    }
}

glib::wrapper! {
    pub struct ResponsePanel(ObjectSubclass<imp::ResponsePanel>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ResponsePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponsePanel {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn present_error(&self, error: RequestError) {
        self.set_response(Option::<Response>::None);
        self.imp().present_error(error);
    }

    pub fn start_request(&self) {
        let imp = self.imp();

        imp.metadata_stack.set_visible_child(&*imp.spinner);
    }

    fn render_response_body_as_text(&self) {
        let buffer = self
            .imp()
            .response_body
            .buffer()
            .downcast::<sourceview5::Buffer>()
            .unwrap();

        let Some(resp) = self.response() else {
            buffer.set_text("");
            return;
        };

        if resp.is_json() {
            #[allow(deprecated)]
            let json = serde_json::from_str(&resp.safe_string())
                .and_then(|text: Value| serde_json::to_string_pretty(&text));
            if let Ok(json) = json {
                buffer.set_text(&json);
            } else {
                #[allow(deprecated)]
                buffer.set_text(&resp.safe_string());
            }
        } else {
            #[allow(deprecated)]
            buffer.set_text(&resp.safe_string());
        }
    }

    pub fn assign_from_response(&self, resp: &Response) {
        self.set_response(Some(resp.clone()));

        let imp = self.imp();
        imp.response_url.set_text(&resp.effective_url());

        let headers = resp.headers().clone();
        imp.response_headers.set_headers(headers);

        let status = format!("• HTTP {}", resp.status_code());
        imp.status_code.set_text(&status);
        imp.status_code.set_visible(true);
        let status_color = match resp.status_code() {
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

        imp.duration.set_text(&format_duration(resp.duration()));
        imp.duration.set_visible(true);

        let size = glib::format_size(resp.size() as u64);
        imp.response_size.set_text(&size);
        imp.response_size.set_visible(true);

        imp.metadata_stack.set_visible_child(&*imp.response_meta);

        let buffer = self
            .imp()
            .response_body
            .buffer()
            .downcast::<sourceview5::Buffer>()
            .unwrap();

        if resp.is_binary() {
            buffer.set_text("");
        } else {
            self.render_response_body_as_text();
        }

        let language = if resp.is_json() {
            LanguageManager::default().language("json")
        } else if resp.is_xml() {
            LanguageManager::default().language("xml")
        } else {
            resp.headers()
                .find_by_name_icase("Content-Type")
                .map(|ctypes| ctypes[0].to_owned())
                .and_then(|ctype| {
                    let ctype = match ctype.split_once(';') {
                        Some((c, _)) => c.to_string(),
                        None => ctype,
                    };
                    LanguageManager::default().guess_language(Option::<PathBuf>::None, Some(&ctype))
                })
        };

        match language {
            Some(language) => buffer.set_language(Some(&language)),
            None => buffer.set_language(None),
        };

        imp.show_response();
    }
}

fn format_duration(duration: u64) -> String {
    if duration >= 1000 {
        // Format as seconds
        let seconds = (duration as f64) / 1000.0;
        let duration = format!("{:.2}", seconds);
        // TRANSLATORS: duration measured in seconds, units in symbol, as in "1.23 s"
        formatx!(gettext("{} s"), duration).unwrap()
    } else {
        // Format as milliseconds.
        // TRANSLATORS: duration measured in milliseconds, as in "234 ms"
        formatx!(gettext("{} ms"), duration).unwrap()
    }
}
