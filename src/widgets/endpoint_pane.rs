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

use glib::{subclass::types::ObjectSubclassIsExt, Object};
use gtk::glib;

use crate::{entities::EndpointData, error::CarteroError};

mod imp {
    use std::cell::RefCell;
    use std::time::Instant;

    use adw::subclass::breakpoint_bin::BreakpointBinImpl;
    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate, StringObject};
    use isahc::RequestExt;

    use crate::app::CarteroApplication;
    use crate::client::{BoundRequest, RequestError};
    use crate::entities::{EndpointData, KeyValue, RequestMethod};
    use crate::error::CarteroError;
    use crate::objects::KeyValueItem;
    use crate::widgets::{ItemPane, KeyValuePane, PayloadTab, ResponsePanel};

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/es/danirod/Cartero/endpoint_pane.ui")]
    #[properties(wrapper_type = super::EndpointPane)]
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
        pub payload_pane: TemplateChild<PayloadTab>,

        #[template_child]
        pub response: TemplateChild<ResponsePanel>,

        #[template_child]
        pub verbs_string_list: TemplateChild<gtk::StringList>,

        #[template_child]
        pub paned: TemplateChild<gtk::Paned>,

        #[property(get, set, nullable)]
        pub item_pane: RefCell<Option<ItemPane>>,
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

    #[glib::derived_properties]
    impl ObjectImpl for EndpointPane {
        fn constructed(&self) {
            self.parent_constructed();

            self.init_dirty_events();
            self.init_settings();
            self.variable_pane.assert_always_placeholder();
            self.header_pane.assert_always_placeholder();
        }
    }

    impl WidgetImpl for EndpointPane {}

    impl BreakpointBinImpl for EndpointPane {}

    #[gtk::template_callbacks]
    impl EndpointPane {
        fn init_dirty_events(&self) {
            self.request_method.connect_selected_item_notify(
                glib::clone!(@weak self as pane => move |_| {
                    let item_pane: &Option<ItemPane> = &pane.item_pane.borrow();
                    let Some(item_pane) = item_pane else {
                        return;
                    };
                    item_pane.set_dirty(true);
                }),
            );

            self.request_url
                .connect_changed(glib::clone!(@weak self as pane => move |_| {
                    let item_pane: &Option<ItemPane> = &pane.item_pane.borrow();
                    let Some(item_pane) = item_pane else {
                        return;
                    };
                    item_pane.set_dirty(true);
                }));

            self.payload_pane
                .connect_changed(glib::clone!(@weak self as pane => move |_| {
                    let item_pane: &Option<ItemPane> = &pane.item_pane.borrow();
                    let Some(item_pane) = item_pane else {
                        return;
                    };
                    item_pane.set_dirty(true);
                }));

            self.header_pane
                .connect_changed(glib::clone!(@weak self as pane => move |_| {
                    let item_pane: &Option<ItemPane> = &pane.item_pane.borrow();
                    let Some(item_pane) = item_pane else {
                        return;
                    };
                    item_pane.set_dirty(true);
                }));
            self.variable_pane
                .connect_changed(glib::clone!(@weak self as pane => move |_| {
                    let item_pane: &Option<ItemPane> = &pane.item_pane.borrow();
                    let Some(item_pane) = item_pane else {
                        return;
                    };
                    item_pane.set_dirty(true);
                }));
        }

        fn init_settings(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();

            settings
                .bind("paned-position", &*self.paned, "position")
                .build();
        }

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
        pub fn assign_request(&self, endpoint: &EndpointData) {
            self.request_url.buffer().set_text(endpoint.url.clone());
            self.set_request_method(endpoint.method.clone());

            let headers: Vec<KeyValueItem> = endpoint
                .headers
                .iter()
                .map(|item| KeyValueItem::from(item.clone()))
                .collect();
            let variables: Vec<KeyValueItem> = endpoint
                .variables
                .iter()
                .map(|item| KeyValueItem::from(item.clone()))
                .collect();
            self.header_pane.set_entries(&headers);
            self.variable_pane.set_entries(&variables);
            self.payload_pane.set_payload(&endpoint.body);
        }

        /// Takes the current state of the pane and extracts it into an Endpoint value.
        pub(super) fn extract_endpoint(&self) -> Result<EndpointData, CarteroError> {
            let header_list = self.header_pane.get_entries();
            let variable_list = self.variable_pane.get_entries();

            let url = String::from(self.request_url.buffer().text());
            let method = self.request_method();

            let headers = header_list
                .iter()
                .map(|pair| KeyValue {
                    name: pair.header_name(),
                    value: pair.header_value(),
                    active: pair.active(),
                    secret: pair.secret(),
                })
                .collect();
            let variables = variable_list
                .iter()
                .map(|pair| KeyValue {
                    name: pair.header_name(),
                    value: pair.header_value(),
                    active: pair.active(),
                    secret: pair.secret(),
                })
                .collect();

            let body = self.payload_pane.payload();
            Ok(EndpointData {
                url,
                method,
                headers,
                variables,
                body,
            })
        }

        /// Executes an HTTP request based on the current contents of the pane.
        pub(super) async fn perform_request(&self) -> Result<(), CarteroError> {
            let request = self.extract_endpoint()?;
            let request = BoundRequest::try_from(request)?;
            let request_obj = isahc::Request::try_from(request)?;

            let start = Instant::now();
            let mut response_obj = request_obj
                .send_async()
                .await
                .map_err(RequestError::NetworkError)?;
            let response = crate::client::extract_isahc_response(&mut response_obj, &start).await?;
            self.response.assign_from_response(&response);
            Ok(())
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
    pub fn assign_endpoint(&self, endpoint: &EndpointData) {
        let imp = self.imp();
        imp.assign_request(endpoint)
    }

    pub fn extract_endpoint(&self) -> Result<EndpointData, CarteroError> {
        let imp = self.imp();
        imp.extract_endpoint()
    }

    /// Executes an HTTP request based on the current contents of the pane.
    ///
    /// TODO: Should actually the EndpointPane do the requests? This method
    /// will probably change once collections are correctly implemented,
    /// since the EndpointPane would be probably bound to an Endpoint object.
    pub async fn perform_request(&self) -> Result<(), CarteroError> {
        let imp = self.imp();
        imp.response.set_spinning(true);
        let outcome = imp.perform_request().await;
        imp.response.set_spinning(false);
        outcome
    }
}
