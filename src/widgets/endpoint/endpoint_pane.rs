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

use adw::prelude::AdwDialogExt;
use gettextrs::gettext;
use glib::{subclass::types::ObjectSubclassIsExt, Object};
use gtk::glib;
use url::form_urlencoded;

use crate::{
    entities::EndpointData,
    error::FileSaveError,
    export::curl::CodeExportService,
    file::{EndpointLoadResult, FileLoadResult},
    widgets::ExportDialog,
};

mod imp {
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};

    use adw::subclass::breakpoint_bin::BreakpointBinImpl;
    use cartero_objects::RequestBodyType;
    use glib::subclass::InitializingObject;
    use glib::{JoinHandle, Properties};
    use gtk::gio::{self, SimpleAction, SimpleActionGroup};
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, ClosureExpression, CompositeTemplate};

    use crate::app::CarteroApplication;
    use crate::entities::{EndpointData, KeyValue};
    use crate::objects::KeyValueItem;
    use crate::widgets::authentication::AuthenticationPane;
    use crate::widgets::endpoint::ResponsePanel;
    use crate::widgets::field::FieldTableListView;
    use crate::widgets::req_body::RequestBodyPane;
    use crate::widgets::{KeyValuePane, MethodDropdown, PayloadTab};

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
        pub header_pane: TemplateChild<FieldTableListView>,

        #[template_child]
        pub variable_pane: TemplateChild<FieldTableListView>,

        #[template_child(id = "method")]
        pub request_method: TemplateChild<MethodDropdown>,

        #[template_child(id = "url")]
        pub request_url: TemplateChild<gtk::Entry>,

        #[template_child]
        body: TemplateChild<RequestBodyPane>,

        #[template_child]
        authentication: TemplateChild<AuthenticationPane>,

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

        #[property(get = Self::has_response_impl)]
        _has_response: RefCell<bool>,

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
                        window.update_url_from_query_params();
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

            self.response.connect_response_notify(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    let obj = pane.obj();
                    obj.notify("has-response");
                }
            ));
        }
    }

    impl WidgetImpl for EndpointPane {}

    impl BreakpointBinImpl for EndpointPane {}

    #[gtk::template_callbacks]
    impl EndpointPane {
        fn has_response_impl(&self) -> bool {
            self.response.response().is_some()
        }

        fn init_actions(&self) {
            let obj = self.obj();

            let action_focus_url = SimpleAction::new("focus-url", None);
            action_focus_url.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.request_url.grab_focus();
                }
            ));

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
                .sync_create()
                .build();

            let action_group = SimpleActionGroup::new();
            action_group.add_action(&action_focus_url);
            action_group.add_action(&action_request);
            action_group.add_action(&action_cancel);
            obj.insert_action_group("endpoint", Some(&action_group));
        }

        fn update_url_from_query_params(&self) {
            println!("TODO: conceal");
        }

        fn update_query_params(&self) -> Result<(), url::ParseError> {
            println!("TODO: conceal");
            Ok(())
        }

        fn init_dirty_events(&self) {
            let obj = self.obj();
            self.request_method
                .connect_request_method_notify(glib::clone!(
                    #[weak]
                    obj,
                    move |_| obj.set_dirty(true)
                ));
            self.request_url.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
            /*self.payload_pane.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));
            self.authentication.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| obj.set_dirty(true)
            ));*/
            /*
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
            */
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
            self.extract_endpoint();
        }

        #[template_callback]
        fn on_url_activated(&self) {
            let _ = self.obj().activate_action("endpoint.request", None);
        }

        /// Sets the value of every widget in the pane into whatever is set by the given endpoint.
        pub fn assign_request(&self, endpoint: &EndpointData) {
            let modern: cartero_objects::Request = endpoint.clone().into();

            self.request_url.buffer().set_text(endpoint.url.clone());
            self.request_method.set_request_method(modern.method());
            self.header_pane.set_table(&modern.headers());
            self.variable_pane.set_table(&modern.variables());
            self.body.set_body(&modern.body());
            self.authentication
                .set_authentication(&modern.authentication());

            // Merge parameters
            let active_params: Vec<KeyValueItem> = self.parameter_pane.get_entries();
            let params = endpoint.parameters.iter().map(KeyValueItem::from);
            let parameters: Vec<KeyValueItem> = active_params.into_iter().chain(params).collect();

            self.parameter_pane.set_entries(&parameters);
        }

        /// Takes the current state of the pane and extracts it into an Endpoint value.
        pub(super) fn extract_endpoint(&self) -> EndpointData {
            let header_list = self.header_pane.table();
            let variable_list = self.variable_pane.table();
            let parameter_list = self.parameter_pane.get_entries();

            let url = String::from(self.request_url.buffer().text());
            let method = self.request_method.request_method().clone().into();

            let parameters = parameter_list
                .iter()
                .map(|pair| KeyValue {
                    name: pair.header_name(),
                    value: pair.header_value(),
                    active: pair.active(),
                    secret: pair.secret(),
                })
                .collect();
            let body = self.body.body();
            let authorization = self.authentication.authentication();
            EndpointData {
                url,
                method,
                parameters,
                headers: header_list.into(),
                variables: variable_list.into(),
                body: body.into(),
                authorization: authorization.into(),
            }
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

        fn request_environment(&self) -> cartero_http::RequestEnvironment {
            let app = CarteroApplication::default();
            let settings = app.settings();
            let validate_tls = settings.boolean("validate-tls");
            let timeout = settings.double("request-timeout");
            let redirects = if settings.boolean("follow-redirects") {
                settings.uint("maximum-redirects") as u64
            } else {
                0
            };
            let config = cartero_http::ClientConfig {
                redirects,
                timeout,
                validate_tls,
            };
            cartero_http::RequestEnvironment { config }
        }

        /// Executes an HTTP request based on the current contents of the pane.
        pub(super) async fn perform_request(&self) {
            let request: cartero_objects::Request = self.extract_endpoint().into();
            let env = self.request_environment();
            let response = cartero_isahc_client::request(&request, &env).await;

            match response {
                Ok(response) => {
                    self.response.assign_from_response(&response);
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            };
        }
    }

    #[cfg(test)]
    mod tests {
        use glib::subclass::types::ObjectSubclassIsExt;
        use gtk::prelude::EditableExt;
        use sourceview5::prelude::ListModelExt;

        use crate::{app::CarteroApplication, objects::KeyValueItem};

        use super::super::EndpointPane;

        fn assert_row(row: &KeyValueItem, name: &str, value: &str, active: bool, secret: bool) {
            assert_eq!(row.header_name(), name);
            assert_eq!(row.header_value(), value);
            assert_eq!(row.active(), active);
            assert_eq!(row.secret(), secret);
        }

        #[gtk::test]
        fn test_setting_the_url_address_updates_params() {
            crate::init_test_resources();
            let _app = CarteroApplication::new();

            let pane = EndpointPane::default();
            let imp = pane.imp();

            /* So far, only the placeholder row in the param pane. */
            assert_eq!(1, imp.parameter_pane.model().n_items());

            imp.request_url
                .set_text("https://www.example.com/foobar.html?a=1&b=2&c=3&d=4");
            assert_eq!(5, imp.parameter_pane.model().n_items());
            assert_row(
                &imp.parameter_pane.item_at(0).unwrap(),
                "a",
                "1",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.item_at(1).unwrap(),
                "b",
                "2",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.item_at(2).unwrap(),
                "c",
                "3",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.item_at(3).unwrap(),
                "d",
                "4",
                true,
                false,
            );
        }

        #[gtk::test]
        fn test_updating_parameter_row_changes_url() {
            crate::init_test_resources();
            let _app = CarteroApplication::new();

            let pane = EndpointPane::default();
            let imp = pane.imp();
            imp.request_url
                .set_text("https://www.example.com/foobar.html?a=1&b=2&c=3&d=4");

            imp.parameter_pane.item_at(2).unwrap().set_header_value("9");
            assert_eq!(
                imp.request_url.text(),
                "https://www.example.com/foobar.html?a=1&b=2&c=9&d=4"
            );
        }

        #[gtk::test]
        fn test_disabling_parameter_row_changes_url() {
            crate::init_test_resources();
            let _app = CarteroApplication::new();

            let pane = EndpointPane::default();
            let imp = pane.imp();
            imp.request_url
                .set_text("https://www.example.com/foobar.html?a=1&b=2&c=3&d=4");

            imp.parameter_pane.item_at(2).unwrap().set_active(false);
            assert_eq!(
                imp.request_url.text(),
                "https://www.example.com/foobar.html?a=1&b=2&d=4"
            );
        }

        #[gtk::test]
        fn test_updating_url_moves_disabled_params_to_bottom() {
            crate::init_test_resources();
            let _app = CarteroApplication::new();

            let pane = EndpointPane::default();
            let imp = pane.imp();
            imp.request_url
                .set_text("https://www.example.com/foobar.html?a=1&b=2&c=3&d=4");
            imp.parameter_pane.item_at(2).unwrap().set_active(false);
            imp.request_url
                .set_text("https://www.example.com/foobar.html?a=1&b=2&d=4&e=5");

            assert_eq!(6, imp.parameter_pane.model().n_items());
            assert_row(
                &imp.parameter_pane.item_at(0).unwrap(),
                "a",
                "1",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.item_at(1).unwrap(),
                "b",
                "2",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.item_at(2).unwrap(),
                "d",
                "4",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.item_at(3).unwrap(),
                "e",
                "5",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.item_at(4).unwrap(),
                "c",
                "3",
                false,
                false,
            );
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

    pub fn export_request(&self, format: &str) {
        let request = self.extract_endpoint();
        let curl = CodeExportService::new(request);

        if let Ok(command) = curl.generate() {
            let buffer = glib::Bytes::from(command.as_bytes());
            let file_format = sourceview5::LanguageManager::default().language("sh");
            let dialog = glib::Object::builder::<ExportDialog>()
                .property("blob", Some(&buffer))
                .property("format", file_format)
                .build();

            let title = match format {
                "curl" => gettext("Export request as cURL"),
                _ => {
                    return;
                }
            };
            dialog.set_title(&title);
            dialog.present(Some(self));
        }
    }
}

fn extract_queryparams(url: &str) -> Vec<(String, String)> {
    let parts = url.split("?").collect::<Vec<&str>>();
    if parts.len() < 2 {
        vec![]
    } else {
        let combined = parts[1..].join("?");
        let params = form_urlencoded::parse(combined.as_bytes());
        params
            .into_iter()
            .map(|(key, value)| (String::from(key), String::from(value)))
            .collect::<_>()
    }
}

fn update_queryparams(url: &str, params: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let url_without_querystring = if url.contains("?") {
        let parts = url.split("?").collect::<Vec<&str>>();
        parts[0]
    } else {
        url
    };

    if params.is_empty() {
        return url_without_querystring.to_string();
    }

    // This is a hand-crafted implementation of x-www-form-urlencoded
    // serializing according to the URL spec from WHATWG section 5.2. However,
    // we skip asserting that the params are scalar strings and we don't
    // percent-encode them, because the URL doesn't have to be valid yet since
    // the output of this string is going to the textarea. There's time to
    // validate and encode the URL when the user sends the request.
    let mut output = String::new();
    for (key, value) in params {
        if !output.is_empty() {
            output.push('&');
        }
        output.push_str(key.as_ref());
        output.push('=');
        output.push_str(value.as_ref());
    }
    [url_without_querystring, &output].join("?")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_queryparams_good_case() {
        let url = "https://www.example.com/index.html?a=1&b=2&c=3";
        let params = vec![("a", "2"), ("d", "4"), ("z", "9")];
        let result = update_queryparams(url, &params);
        assert_eq!(result, "https://www.example.com/index.html?a=2&d=4&z=9");
    }

    #[test]
    fn update_queryparams_empty() {
        let empty: &[(&str, &str)] = &[];
        let result = update_queryparams("", &empty);
        assert_eq!(result, "");
    }

    #[test]
    fn update_queryparams_valid_url_without_qs_and_empty_params() {
        let empty: &[(&str, &str)] = &[];
        let result = update_queryparams("https://www.example.com/index.html", &empty);
        assert_eq!(result, "https://www.example.com/index.html");
    }

    #[test]
    fn update_queryparams_removes_querystring_if_empty_params() {
        let empty: &[(&str, &str)] = &[];
        let result = update_queryparams("https://www.example.com/index.html?a=1&b=2", &empty);
        assert_eq!(result, "https://www.example.com/index.html");
    }

    #[test]
    fn update_queryparams_adds_queryparams() {
        let url = "https://www.example.com/index.html";
        let params = vec![("a", "2"), ("d", "4"), ("z", "9")];
        let result = update_queryparams(url, &params);
        assert_eq!(result, "https://www.example.com/index.html?a=2&d=4&z=9");
    }

    #[test]
    fn update_queryparams_adds_queryparams_partial_urls() {
        let url = "/api/v1/users";
        let params = vec![("page", "2"), ("sort", "-created")];
        let result = update_queryparams(url, &params);
        assert_eq!(result, "/api/v1/users?page=2&sort=-created")
    }

    #[test]
    fn update_queryparams_updates_queryparams() {
        let url = "http://{{ROOT}}/users?page=1&sort=-created";
        let params = vec![("page", "2")];
        let result = update_queryparams(url, &params);
        assert_eq!(result, "http://{{ROOT}}/users?page=2")
    }

    #[test]
    fn update_queryparams_updates_queryparams_with_variables() {
        let url = "http://www.example.com/api/users?page=1&sort=-created";
        let params = vec![("page", "{{PID}}"), ("sort", "created")];
        let result = update_queryparams(url, &params);
        assert_eq!(
            result,
            "http://www.example.com/api/users?page={{PID}}&sort=created"
        );
    }

    #[test]
    fn extract_queryparams_no_params() {
        let url = "https://www.example.com/foobar.html";
        let result = extract_queryparams(url);
        assert!(result.is_empty());
    }

    #[test]
    fn extract_queryparams_straight_case() {
        let url = "https://www.example.com/foobar.html?a=1&b=2&c=3&d=4";
        let expected = vec![("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")];
        let result = extract_queryparams(&url);
        assert_eq!(result.len(), expected.len());
        for (result, expect) in result.iter().zip(expected) {
            assert_eq!(result.0, expect.0);
            assert_eq!(result.1, expect.1);
        }
    }

    #[test]
    fn extract_queryparams_multiple_questions() {
        // You should probably URL-encode this, tho.
        let url =
            "https://www.example.com/foobar.html?question1=where?&question2=how?&question3=when?";
        let expected = vec![
            ("question1", "where?"),
            ("question2", "how?"),
            ("question3", "when?"),
        ];
        let result = extract_queryparams(&url);
        assert_eq!(result.len(), expected.len());
        for (result, expect) in result.iter().zip(expected) {
            assert_eq!(result.0, expect.0);
            assert_eq!(result.1, expect.1);
        }
    }

    #[test]
    fn extract_queryparams_variables_before() {
        // It's going to do what it can.
        let url = "https://{{ROOT}}/users?limit=20&offset=30";
        let expected = vec![("limit", "20"), ("offset", "30")];
        let result = extract_queryparams(&url);
        for (result, expect) in result.iter().zip(expected) {
            assert_eq!(result.0, expect.0);
            assert_eq!(result.1, expect.1);
        }
    }

    #[test]
    fn extract_queryparams_variables_inside() {
        let url = "https://www.example.com/users?limit=10&page={{PID}}&order=desc";
        let expected = vec![("limit", "10"), ("page", "{{PID}}"), ("order", "desc")];
        let result = extract_queryparams(&url);
        for (result, expect) in result.iter().zip(expected) {
            assert_eq!(result.0, expect.0);
            assert_eq!(result.1, expect.1);
        }
    }

    #[test]
    fn extract_queryparams_variables_inappropiate() {
        // Imagine that PREFIX = www.example.com/users?page=
        let url = "https://{{PREFIX}}2";
        let result = extract_queryparams(&url);
        assert!(result.is_empty()); // yeah, I can't do magic here
    }
}
