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

use crate::app::CarteroApplication;
use glib::Object;
use gtk4::{gio, glib};

mod imp {
    use glib::GString;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    use gtk4::gio::ActionEntry;
    use gtk4::FileChooserAction;
    use gtk4::StringObject;
    use gtk4::Window;
    use isahc::RequestExt;

    use crate::client::Request;
    use crate::client::RequestError;
    use crate::client::RequestMethod;
    use crate::client::Response;
    use crate::widgets::*;
    use glib::subclass::InitializingObject;
    use gtk4::{
        subclass::{
            application_window::ApplicationWindowImpl, widget::WidgetImpl, window::WindowImpl,
        },
        CompositeTemplate, TemplateChild,
    };

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/main_window.ui")]
    pub struct CarteroWindow {
        #[template_child(id = "send")]
        pub send_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub header_pane: TemplateChild<RequestHeaderPane>,

        #[template_child(id = "method")]
        pub request_method: TemplateChild<gtk4::DropDown>,

        #[template_child(id = "url")]
        pub request_url: TemplateChild<gtk4::Entry>,

        #[template_child]
        pub request_body: TemplateChild<sourceview5::View>,

        #[template_child]
        pub response: TemplateChild<ResponsePanel>,

        #[template_child]
        pub verbs_string_list: TemplateChild<gtk4::StringList>,
    }

    impl CarteroWindow {
        fn request_method(&self) -> GString {
            self.request_method
                .selected_item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string()
        }

        fn set_request_method(&self, rm: RequestMethod) {
            let verb_to_find = String::from(rm);
            let element_count = self.request_method.model().unwrap().n_items();
            let target_position = (0..element_count).find(|i| {
                if let Some(verb) = self.verbs_string_list.string(*i) {
                    if verb.to_string() == verb_to_find {
                        return true;
                    }
                }
                false
            });
            if let Some(pos) = target_position {
                self.request_method.set_selected(pos);
            }
        }

        // Convert from a Request object into UI state
        fn assign_request(&self, req: &Request) {
            self.request_url.buffer().set_text(req.url.clone());
            self.set_request_method(req.method.clone());
            self.header_pane.set_headers(&req.headers);
            let body = String::from_utf8_lossy(&req.body).to_owned();
            self.request_body.buffer().set_text(&body);
        }

        // Convert from UI state into a Request object
        fn extract_request(&self) -> Result<Request, RequestError> {
            let url = String::from(self.request_url.buffer().text());
            let method = RequestMethod::try_from(self.request_method().as_str())?;
            let headers = {
                let vector = self.header_pane.get_headers();
                vector
                    .iter()
                    .filter(|h| h.is_usable())
                    .map(|h| (h.header_name(), h.header_value()))
                    .collect()
            };
            let body = {
                let buffer = self.request_body.buffer();
                let (start, end) = buffer.bounds();
                let text = buffer.text(&start, &end, true);
                Vec::from(text.as_bytes())
            };
            Ok(Request::new(url, method, headers, body))
        }

        async fn trigger_save(&self) {
            let dialog = gtk4::FileDialog::builder()
                .accept_label("Guardar")
                .title("Guardar petición")
                .modal(true)
                .build();
            let result = dialog
                .save_future(gtk4::Window::NONE)
                .await
                .map_err(|e| println!("{:?}", e));
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let request = self.extract_request().unwrap();
                    let serialized_payload = crate::file::store_toml(&request).unwrap();
                    let _ = crate::file::write_file(&path, &serialized_payload);
                }
            }
        }

        fn perform_request(&self) {
            let request = self.extract_request().unwrap();
            let request_obj = isahc::Request::try_from(request).unwrap();
            let mut response_obj = request_obj.send().unwrap();
            let response = Response::try_from(&mut response_obj).unwrap();
            self.response.assign_from_response(&response);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroWindow {
        const NAME: &'static str = "CarteroWindow";
        type Type = super::CarteroWindow;
        type ParentType = gtk4::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            RequestHeaderRow::static_type();
            RequestHeaderPane::static_type();
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CarteroWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let action_request = ActionEntry::builder("request")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    window.perform_request();
                }))
                .build();
            let action_save = ActionEntry::builder("save")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        window.trigger_save().await;
                    }));
                }))
                .build();

            let obj = self.obj();
            obj.add_action_entries([action_request, action_save]);
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {}

    impl ApplicationWindowImpl for CarteroWindow {}
}

glib::wrapper! {
    pub struct CarteroWindow(ObjectSubclass<imp::CarteroWindow>)
        @extends gtk4::Widget, gtk4::Window, gtk4::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CarteroWindow {
    pub fn new(app: &CarteroApplication) -> Self {
        Object::builder().property("application", Some(app)).build()
    }
}
