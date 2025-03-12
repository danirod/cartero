// Copyright 2024-2025 the Cartero authors
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

use crate::{
    entities::EndpointData,
    error::FileSaveError,
    file::{EndpointLoadResult, FileLoadResult},
};

mod imp {
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    use adw::subclass::breakpoint_bin::BreakpointBinImpl;
    use glib::subclass::InitializingObject;
    use glib::{JoinHandle, Properties};
    use gtk::gio::{self, SimpleAction, SimpleActionGroup};
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, ClosureExpression, CompositeTemplate};
    use isahc::RequestExt;
    use url::Url;

    use crate::app::CarteroApplication;
    use crate::client::BoundRequest;
    use crate::entities::{EndpointData, KeyValue, RequestExportType, ResponseData};
    use crate::error::{RequestError, RequestPreconditionError};
    use crate::objects::KeyValueItem;
    use crate::widgets::{
        ExportTab, ExportType, KeyValuePane, MethodDropdown, PayloadTab, ResponsePanel,
    };

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/es/danirod/Cartero/endpoint_pane.ui")]
    #[properties(wrapper_type = super::EndpointPane)]
    pub struct EndpointPane {
        #[template_child(id = "send")]
        pub send_button: TemplateChild<gtk::Button>,

        #[template_child(id = "cancel")]
        cancel_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub parameter_pane: TemplateChild<KeyValuePane>,

        #[template_child]
        pub header_pane: TemplateChild<KeyValuePane>,

        #[template_child]
        pub variable_pane: TemplateChild<KeyValuePane>,

        #[template_child(id = "method")]
        pub request_method: TemplateChild<MethodDropdown>,

        #[template_child(id = "url")]
        pub request_url: TemplateChild<gtk::Entry>,

        #[template_child]
        pub payload_pane: TemplateChild<PayloadTab>,

        #[template_child]
        pub export_pane: TemplateChild<ExportTab>,

        #[template_child]
        pub response: TemplateChild<ResponsePanel>,

        #[template_child]
        pub paned: TemplateChild<gtk::Paned>,

        #[property(get, set, name = "read-only")]
        read_only: RefCell<bool>,

        #[property(get, set, nullable)]
        file: RefCell<Option<gio::File>>,

        #[property(get, set)]
        dirty: RefCell<bool>,

        // Busy requesting
        #[property(get, set)]
        busy: RefCell<bool>,

        request_thread: Arc<RefCell<Option<JoinHandle<()>>>>,

        variable_changing: Arc<Mutex<bool>>,
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
            self.init_actions();

            self.variable_pane.assert_always_placeholder();
            self.header_pane.assert_always_placeholder();
            self.parameter_pane.assert_always_placeholder();

            let url_arc = self.variable_changing.clone();
            self.request_url.connect_changed(glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |_| {
                    // It is important to allow the redundant pattern matching because
                    // is_ok() does not capture the mutex and will cause sync issues.
                    #[allow(clippy::redundant_pattern_matching)]
                    if let Ok(_) = url_arc.try_lock() {
                        if let Err(err) = window.update_query_params() {
                            println!("{err}");
                        }
                    }
                }
            ));

            let parameter_arc = self.variable_changing.clone();
            self.parameter_pane.connect_changed(glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |_| {
                    // It is important to allow the redundant pattern matching because
                    // is_ok() does not capture the mutex and will cause sync issues.
                    #[allow(clippy::redundant_pattern_matching)]
                    if let Ok(_) = parameter_arc.try_lock() {
                        if let Err(err) = window.update_url_from_query_params() {
                            println!("{err}");
                        }
                    }
                }
            ));

            // update export pane data when user selects another option in the combo box.
            self.export_pane.connect_changed(glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |_| {
                    if window.export_pane.imp().export_type() == ExportType::Curl {
                        let data = window.extract_endpoint();
                        window.export_pane_load_endpoint_data(&data);
                    }
                }
            ));

            // Mark the window as busy when actually busy.
            let obj = self.obj();
            obj.property_expression("busy")
                .chain_closure::<gtk::gdk::Cursor>(glib::closure!(
                    |_: &super::EndpointPane, busy: bool| {
                        if busy {
                            gtk::gdk::Cursor::from_name("wait", None)
                        } else {
                            None
                        }
                    }
                ))
                .bind(&*obj, "cursor", Some(&*obj));

            self.configure_export_pane_bindings();
        }
    }

    impl WidgetImpl for EndpointPane {}

    impl BreakpointBinImpl for EndpointPane {}

    #[gtk::template_callbacks]
    impl EndpointPane {
        fn init_actions(&self) {
            let obj = self.obj();

            let action_request = SimpleAction::new("request", None);
            action_request.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.action_perform_request();
                }
            ));

            let action_cancel = SimpleAction::new("cancel", None);
            action_cancel.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.action_cancel_request();
                }
            ));

            /* The action should only be enabled if there is an URL set and if not busy. */
            let text = self.request_url.property_expression("text");
            let busy = obj.property_expression("busy");

            ClosureExpression::new::<bool>(
                &[&text, &busy],
                glib::closure!(|_: &super::EndpointPane, text: &str, busy: bool| {
                    // The question is: is enabled? So returns true unless empty or busy.
                    !(busy || text.is_empty())
                }),
            )
            .bind(&action_request, "enabled", Some(&*obj));

            /* The cancel should only be possible if currently busy. */
            busy.bind(&action_cancel, "enabled", gtk::Widget::NONE);

            /* Also, bind the visibility of the cancel button to whether the action is enabled. */
            action_cancel
                .bind_property("enabled", &*self.cancel_button, "visible")
                .build();

            let action_group = SimpleActionGroup::new();
            action_group.add_action(&action_request);
            action_group.add_action(&action_cancel);
            obj.insert_action_group("endpoint", Some(&action_group));
        }

        fn update_url_from_query_params(&self) -> Result<(), url::ParseError> {
            let table = self.parameter_pane.get_entries();

            let parsed_url = self.request_url.text().to_string();
            let mut url = Url::parse(&parsed_url)?;
            {
                let mut pairs = url.query_pairs_mut();
                pairs.clear();
                for item in table {
                    if item.active() {
                        let key = item.header_name();
                        let value = item.header_value();
                        pairs.append_pair(&key, &value);
                    }
                }
            }
            self.request_url.set_text(url.as_str());
            Ok(())
        }

        fn update_query_params(&self) -> Result<(), url::ParseError> {
            let parsed_url = self.request_url.text().to_string();
            let url = Url::parse(&parsed_url)?;
            let new_query_pairs = url.query_pairs();
            let mut new_query_entries: Vec<KeyValueItem> = new_query_pairs
                .map(|(key, value)| {
                    let key = String::from(key);
                    let value = String::from(value);
                    let entry = KeyValue::from((key, value));
                    let value = KeyValueItem::from(entry);
                    value.set_active(true);
                    value.set_secret(false);
                    value
                })
                .collect();

            let old_entries = self.parameter_pane.get_entries();
            let old_entries: Vec<KeyValueItem> = old_entries
                .into_iter()
                .filter(|entry| !entry.active())
                .collect();
            new_query_entries.extend(old_entries);

            self.parameter_pane.set_entries(&new_query_entries);
            Ok(())
        }

        fn init_dirty_events(&self) {
            let obj = self.obj();
            self.request_method.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
            self.request_url.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
            self.payload_pane.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
            self.export_pane.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
            self.header_pane.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
            self.variable_pane.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
        }

        fn init_settings(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();
            let initial_position = SettingsExtManual::get(settings, "paned-position");
            self.paned.set_position(initial_position);

            self.paned.connect_position_notify(glib::clone!(
                #[weak]
                settings,
                move |paned| {
                    let new_position = paned.position();
                    let _ = settings.set("paned-position", new_position);
                }
            ));
        }

        #[template_callback]
        fn on_url_changed(&self) {
            let data = self.extract_endpoint();
            self.export_pane_load_endpoint_data(&data);
        }

        #[template_callback]
        fn on_url_activated(&self) {
            let _ = self.obj().activate_action("endpoint.request", None);
        }

        /// Loads data for the export pane module by using an `EndpointData` structure.
        fn export_pane_load_endpoint_data(&self, endpoint: &EndpointData) {
            let req_export_type = self.export_pane.request_export_type();

            if let RequestExportType::None = req_export_type {
                return;
            }

            if let RequestExportType::Curl(_) = req_export_type {
                self.export_pane
                    .set_request_export_type(&RequestExportType::Curl(endpoint.clone()));
            }
        }

        /// Retrieves `EndpointData` and builds a new state for the export request module.
        fn update_export_pane(&self) {
            let data = self.extract_endpoint();
            self.export_pane_load_endpoint_data(&data);
        }

        /// Connect ourself to every widget in order to pass new data and rehydrate the
        /// export pane module so it gets realtime, maybe we should consider doing some
        /// kind of reactive bindings?
        fn configure_export_pane_bindings(&self) {
            self.request_method.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| pane.update_export_pane()
            ));
            self.request_url.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| pane.update_export_pane()
            ));
            self.payload_pane.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| pane.update_export_pane()
            ));
            self.export_pane.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| pane.update_export_pane()
            ));
            self.header_pane.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| pane.update_export_pane()
            ));
            self.variable_pane.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| pane.update_export_pane()
            ));
        }

        /// Sets the value of every widget in the pane into whatever is set by the given endpoint.
        pub fn assign_request(&self, endpoint: &EndpointData) {
            self.request_url.buffer().set_text(endpoint.url.clone());
            self.request_method
                .set_request_method(endpoint.method.clone());
            let headers: Vec<KeyValueItem> =
                endpoint.headers.iter().map(KeyValueItem::from).collect();
            let variables: Vec<KeyValueItem> =
                endpoint.variables.iter().map(KeyValueItem::from).collect();
            self.header_pane.set_entries(&headers);
            self.variable_pane.set_entries(&variables);
            self.payload_pane.set_payload(&endpoint.body);
            self.export_pane_load_endpoint_data(endpoint);

            // Merge parameters
            let active_params: Vec<KeyValueItem> = self.parameter_pane.get_entries();
            let params = endpoint.parameters.iter().map(KeyValueItem::from);
            let parameters: Vec<KeyValueItem> = active_params.into_iter().chain(params).collect();

            self.parameter_pane.set_entries(&parameters);
        }

        /// Takes the current state of the pane and extracts it into an Endpoint value.
        pub(super) fn extract_endpoint(&self) -> EndpointData {
            let header_list = self.header_pane.get_entries();
            let variable_list = self.variable_pane.get_entries();
            let parameter_list = self.parameter_pane.get_entries();

            let url = String::from(self.request_url.buffer().text());
            let method = self.request_method.request_method();

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
            let parameters = parameter_list
                .iter()
                .map(|pair| KeyValue {
                    name: pair.header_name(),
                    value: pair.header_value(),
                    active: pair.active(),
                    secret: pair.secret(),
                })
                .collect();
            let body = self.payload_pane.payload();
            EndpointData {
                url,
                method,
                parameters,
                headers,
                variables,
                body,
            }
        }

        fn bind_request(&self) -> Result<BoundRequest, RequestPreconditionError> {
            let request = self.extract_endpoint();
            let request = BoundRequest::try_from(request);

            // A special case before giving up: if the protocol is not specified, add it.
            match request {
                Err(RequestPreconditionError::MissingProtocol) => {
                    // The URL is not considered an absolute URL, missing protocol.
                    let url_field = self.request_url.text().to_string();
                    let url_field = format!("http://{}", url_field);
                    self.request_url.set_text(&url_field);

                    // Now try again. If it fails again, just bail with the original error.
                    let request = self.extract_endpoint();
                    BoundRequest::try_from(request)
                }
                any => any,
            }
        }

        async fn execute_request(
            &self,
            request: isahc::http::Request<Vec<u8>>,
        ) -> Result<ResponseData, RequestError> {
            let start = Instant::now();
            let mut response = request
                .send_async()
                .await
                .map_err(RequestError::NetworkError)?;
            let response = crate::client::extract_isahc_response(&mut response, &start).await?;
            Ok(response)
        }

        fn action_cancel_request(&self) {
            let obj = self.obj();

            {
                let maybe_request_thread = self.request_thread.borrow();
                if let Some(ref thread_ref) = *maybe_request_thread {
                    thread_ref.abort();

                    /* reset the user interface state. */
                    self.response.set_spinning(false);
                    obj.set_read_only(false);
                    obj.set_busy(false);
                }
            }
            self.request_thread.replace(None);
        }

        fn action_perform_request(&self) {
            let thread_ref = glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                async move {
                    let obj = imp.obj();

                    /* prelude */
                    obj.set_busy(true);
                    obj.set_read_only(true);
                    imp.response.set_spinning(true);

                    imp.perform_request().await;

                    /* restore */
                    imp.response.set_spinning(false);
                    obj.set_read_only(false);
                    obj.set_busy(false);

                    imp.request_thread.replace(None);
                }
            ));

            self.request_thread.replace(Some(thread_ref));
        }

        /// Executes an HTTP request based on the current contents of the pane.
        pub(super) async fn perform_request(&self) {
            let bind_request = match self.bind_request() {
                Ok(bind) => bind,
                Err(e) => {
                    self.response.show_precondition_error(e);
                    return;
                }
            };

            let client_obj = match crate::client::build_request(&bind_request) {
                Ok(request) => request,
                Err(e) => {
                    self.response.show_request_build_error(e);
                    return;
                }
            };

            match self.execute_request(client_obj).await {
                Ok(data) => self.response.assign_from_response(&data),
                Err(e) => self.response.show_request_error(e),
            };
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
    pub fn new() -> Self {
        // TODO: Accept additional initial state maybe?
        Object::builder().build()
    }

    /// Updates the contents of the widget so that they reflect the endpoint data.
    ///
    /// TODO: Should enable a binding system?
    pub fn assign_endpoint(&self, endpoint: &EndpointData) {
        let imp = self.imp();
        imp.assign_request(endpoint)
    }

    pub fn extract_endpoint(&self) -> EndpointData {
        let imp = self.imp();
        imp.extract_endpoint()
    }

    pub async fn load(&self) -> impl FileLoadResult {
        match self.file() {
            Some(file) => {
                let result = crate::file::read_endpoint(&file).await;
                if let Some(endpoint) = result.endpoint() {
                    self.assign_endpoint(&endpoint);
                    self.set_dirty(false);
                }
                result
            }
            None => EndpointLoadResult::anonymous(),
        }
    }

    pub async fn save(&self) -> Result<(), FileSaveError> {
        match self.file() {
            Some(file) => {
                let endpoint = self.extract_endpoint();
                let result = crate::file::write_endpoint(&file, &endpoint).await;
                if let Ok(()) = result {
                    self.set_dirty(false);
                }
                result
            }
            None => Err(FileSaveError::AnonymousPane),
        }
    }
}
