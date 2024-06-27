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

use glib::{subclass::types::ObjectSubclassIsExt, value::ToValue, Object};
use gtk::{
    gio::{Settings, SettingsBindFlags},
    glib,
    prelude::SettingsExtManual,
    WrapMode,
};

use crate::{error::CarteroError, objects::Endpoint};

mod imp {
    use std::collections::HashMap;

    use adw::subclass::breakpoint_bin::BreakpointBinImpl;
    use adw::Banner;
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate, StringObject};
    use isahc::RequestExt;

    use crate::client::{Request, RequestError, RequestMethod, Response};
    use crate::error::CarteroError;
    use crate::objects::Endpoint;
    use crate::widgets::{KeyValuePane, ResponsePanel};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/endpoint_pane.ui")]
    pub struct EndpointPane {
        #[template_child(id = "send")]
        pub send_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub header_pane: TemplateChild<KeyValuePane>,

        #[template_child]
        pub variable_pane: TemplateChild<KeyValuePane>,

        #[template_child(id = "method")]
        pub request_method: TemplateChild<gtk::DropDown>,

        #[template_child(id = "url")]
        pub request_url: TemplateChild<gtk::Entry>,

        #[template_child]
        pub request_body: TemplateChild<sourceview5::View>,

        #[template_child]
        pub response: TemplateChild<ResponsePanel>,

        #[template_child]
        pub verbs_string_list: TemplateChild<gtk::StringList>,

        #[template_child]
        banner: TemplateChild<adw::Banner>,

        #[template_child]
        pub paned: TemplateChild<gtk::Paned>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EndpointPane {
        const NAME: &'static str = "CarteroEndpointPane";
        type Type = super::EndpointPane;
        type ParentType = adw::BreakpointBin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EndpointPane {}

    impl WidgetImpl for EndpointPane {}

    impl BreakpointBinImpl for EndpointPane {}

    #[gtk::template_callbacks]
    impl EndpointPane {
        /// Syncs whether the Send button can be clicked based on whether the request is formed.
        ///
        /// For a request to be formed, an URL has to be set. You cannot submit a request if
        /// you haven't introduced an URL into the corresponding entry field. Every other field
        /// can be blank.
        fn update_send_button_sensitivity(&self) {
            let empty = self.request_url.buffer().text().is_empty();
            self.send_button.set_sensitive(!empty);
        }

        #[template_callback]
        fn on_url_changed(&self) {
            self.update_send_button_sensitivity();
        }

        #[template_callback]
        fn on_close_banner(banner: &Banner) {
            banner.set_revealed(false);
        }

        /// Decodes the HTTP method that has been picked by the user in the dropdown.
        fn request_method(&self) -> RequestMethod {
            let method = self
                .request_method
                .selected_item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string();
            // Note: we should probably be safe from unwrapping here, since it would
            // be impossible to have a method that is not an acceptable value without
            // completely hacking and wrecking the user interface.
            RequestMethod::try_from(method.as_str()).unwrap()
        }

        /// Sets the currently picked HTTP method for the method dropdown.
        ///
        /// TODO: This method should probably be part of its own widget.
        fn set_request_method(&self, rm: RequestMethod) {
            let verb_to_find = String::from(rm);
            let element_count = self.request_method.model().unwrap().n_items();
            let target_position = (0..element_count).find(|i| {
                if let Some(verb) = self.verbs_string_list.string(*i) {
                    if verb == verb_to_find {
                        return true;
                    }
                }
                false
            });
            if let Some(pos) = target_position {
                self.request_method.set_selected(pos);
            }
        }

        /// Sets the value of every widget in the pane into whatever is set by the given endpoint.
        pub fn assign_request(&self, ep: Endpoint) {
            let Endpoint(req, variables) = ep;
            self.request_url.buffer().set_text(req.url.clone());
            self.set_request_method(req.method.clone());
            self.header_pane.set_entries(&req.headers);
            self.variable_pane.set_entries(&variables);
            let body = String::from_utf8_lossy(&req.body);
            self.request_body.buffer().set_text(&body);
        }

        fn extract_request(&self) -> Result<Request, CarteroError> {
            let header_list = self.header_pane.get_entries();

            let url = String::from(self.request_url.buffer().text());
            let method = self.request_method();
            let headers = header_list
                .iter()
                .filter(|pair| pair.is_usable())
                .map(|pair| (pair.header_name(), pair.header_value()))
                .collect();

            let body = {
                let buffer = self.request_body.buffer();
                let (start, end) = buffer.bounds();
                let text = buffer.text(&start, &end, true);
                Vec::from(text)
            };
            Ok(Request::new(url, method, headers, body))
        }

        fn extract_variables(&self) -> HashMap<String, String> {
            let variables = self.variable_pane.get_entries();
            variables
                .iter()
                .filter(|v| v.is_usable())
                .map(|v| (v.header_name(), v.header_value()))
                .collect()
        }

        /// Takes the current state of the pane and extracts it into an Endpoint value.
        pub(super) fn extract_endpoint(&self) -> Result<Endpoint, CarteroError> {
            let request = self.extract_request()?;
            let variables = self.extract_variables();
            Ok(Endpoint(request, variables))
        }

        /// Executes an HTTP request based on the current contents of the pane.
        pub(super) fn perform_request(&self) -> Result<(), CarteroError> {
            let request = self.extract_request()?;
            let request = request.bind(&self.extract_variables())?;
            let request_obj = isahc::Request::try_from(request)?;
            let mut response_obj = request_obj.send().map_err(RequestError::NetworkError)?;
            let response = Response::try_from(&mut response_obj)?;
            self.response.assign_from_response(&response);
            self.hide_banner();
            Ok(())
        }

        pub(super) fn show_banner(&self, message: &str) {
            self.banner.set_title(message);
            self.banner.set_revealed(true);
        }

        pub(super) fn hide_banner(&self) {
            self.banner.set_title("");
            self.banner.set_revealed(false);
        }
    }
}

glib::wrapper! {
    pub struct EndpointPane(ObjectSubclass<imp::EndpointPane>)
        @extends gtk::Widget, gtk::Box;
}

impl Default for EndpointPane {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl EndpointPane {
    /// Updates the contents of the widget so that they reflect the endpoint data.
    ///
    /// TODO: Should enable a binding system?
    pub fn assign_endpoint(&self, endpoint: Endpoint) {
        let imp = self.imp();
        imp.assign_request(endpoint)
    }

    pub fn extract_endpoint(&self) -> Result<Endpoint, CarteroError> {
        let imp = self.imp();
        imp.extract_endpoint()
    }

    /// Executes an HTTP request based on the current contents of the pane.
    ///
    /// TODO: Should actually the EndpointPane do the requests? This method
    /// will probably change once collections are correctly implemented,
    /// since the EndpointPane would be probably bound to an Endpoint object.
    pub fn perform_request(&self) -> Result<(), CarteroError> {
        let imp = self.imp();
        imp.perform_request()
    }

    /// Shows the error message revealer to disclose an error message.
    pub fn show_banner(&self, message: &str) {
        let imp = self.imp();
        imp.show_banner(message)
    }

    /// Hides the error message revealer if previously was visible.
    pub fn hide_banner(&self) {
        let imp = self.imp();
        imp.hide_banner()
    }

    /// Bind the widgets in this pane to the application settings.
    ///
    /// This method has to be called during instantiation so that the application
    /// settings can be bound, the widget can use the values currently defined in
    /// the settings, and updating the settings automatically refreshes the pane.
    pub fn bind_settings(&self, settings: &Settings) {
        let imp = self.imp();
        imp.response.get().assign_settings(settings);

        let body = imp.request_body.get();
        settings
            .bind("body-wrap", &body, "wrap-mode")
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
            .bind("paned-position", &*imp.paned, "position")
            .build();
    }
}
