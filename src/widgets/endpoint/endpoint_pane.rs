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
use glib::object::CastNone;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::gio::{self, FileCreateFlags};
use gtk::glib;
use gtk::prelude::WidgetExt;
use sourceview5::prelude::FileExtManual;
use url::form_urlencoded;

use crate::widgets::shell::BasePane;
use crate::{
    export::curl::CodeExportService,
    widgets::{file_dialogs, ExportDialog},
};

mod imp {
    use std::cell::{OnceCell, RefCell};
    use std::sync::{Arc, Mutex};

    use adw::subclass::breakpoint_bin::BreakpointBinImpl;
    use cartero_http::RequestError;
    use cartero_objects::{Field, Request, Response};
    use glib::subclass::InitializingObject;
    use glib::{JoinHandle, Properties};
    use gtk::gio::{self, Cancellable, FileCreateFlags, SimpleAction, SimpleActionGroup};
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, ClosureExpression, CompositeTemplate};

    use crate::app::CarteroApplication;
    use crate::interop::{InnerError, LoadResult, SaveResult};
    use crate::widgets::authentication::AuthenticationPane;
    use crate::widgets::endpoint::ResponsePanel;
    use crate::widgets::field::FieldTableListView;
    use crate::widgets::req_body::RequestBodyPane;
    use crate::widgets::shell::BasePaneImpl;
    use crate::widgets::MethodDropdown;

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/es/danirod/Cartero/endpoint_pane.ui")]
    #[properties(wrapper_type = super::EndpointPane)]
    pub struct EndpointPane {
        #[template_child(id = "cancel")]
        cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        parameter_pane: TemplateChild<FieldTableListView>,
        #[template_child]
        header_pane: TemplateChild<FieldTableListView>,
        #[template_child]
        variable_pane: TemplateChild<FieldTableListView>,
        #[template_child]
        request_method: TemplateChild<MethodDropdown>,
        #[template_child]
        request_url: TemplateChild<gtk::Entry>,
        #[template_child]
        body_pane: TemplateChild<RequestBodyPane>,
        #[template_child]
        authentication_pane: TemplateChild<AuthenticationPane>,
        #[template_child]
        response_pane: TemplateChild<ResponsePanel>,
        #[template_child]
        paned: TemplateChild<gtk::Paned>,

        #[property(get, set, name = "read-only")]
        read_only: RefCell<bool>,

        #[property(get, set)]
        request: RefCell<Request>,
        request_signal_group: OnceCell<glib::SignalGroup>,

        #[property(get)]
        request_binding_group: RefCell<glib::BindingGroup>,

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
        type ParentType = crate::widgets::shell::BasePane;

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

            self.init_request_binding_group();

            self.init_dirty_events();
            self.init_settings();
            self.init_actions();

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

            self.response_pane.connect_response_notify(glib::clone!(
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

    impl BasePaneImpl for EndpointPane {
        fn lookup_action(&self, name: &str) -> Option<gio::Action> {
            self.obj().lookup_action(name)
        }

        fn load(&self) -> LoadResult {
            let Some(file) = self.obj().file() else {
                return LoadResult::Anonymous;
            };

            let contents = file.load_contents(gio::Cancellable::NONE);
            match contents {
                Ok((contents, _)) => {
                    let input = String::from_utf8_lossy(&contents).to_string();
                    match cartero_file_format::deserialize_request(&input) {
                        Ok(result) => {
                            let request = result.object();
                            self.obj().set_request(request);

                            let warnings = result.warnings();
                            if warnings.is_empty() {
                                LoadResult::Successful
                            } else {
                                LoadResult::Warning(warnings)
                            }
                        }
                        Err(e) => LoadResult::Error(InnerError::InteropError(e)),
                    }
                }
                Err(e) => LoadResult::Error(InnerError::GlibError(e)),
            }
        }

        fn save(&self) -> SaveResult {
            let Some(file) = self.obj().file() else {
                return SaveResult::Anonymous;
            };

            let create_file_backup = {
                let app = CarteroApplication::default();
                let settings = app.settings();
                settings.get::<bool>("create-backup-files")
            };

            let request = self.obj().request();
            match cartero_file_format::serialize_request(&request) {
                Ok(contents) => {
                    let use_backups = create_file_backup;
                    let saved = file.replace_contents(
                        contents.as_bytes(),
                        None,
                        use_backups,
                        FileCreateFlags::NONE,
                        Cancellable::NONE,
                    );

                    match saved {
                        Ok(_) => {
                            self.obj().set_dirty(false);
                            SaveResult::Successful
                        }
                        Err(e) => SaveResult::Error(InnerError::GlibError(e)),
                    }
                }
                Err(e) => SaveResult::Error(InnerError::InteropError(e)),
            }
        }
    }

    #[gtk::template_callbacks]
    impl EndpointPane {
        fn init_request_binding_group(&self) {
            let binding_group = self.request_binding_group.borrow();

            binding_group
                .bind("url", &*self.request_url, "text")
                .bidirectional()
                .sync_create()
                .build();
            binding_group
                .bind("method", &*self.request_method, "request-method")
                .bidirectional()
                .sync_create()
                .build();
            binding_group
                .bind("params", &*self.parameter_pane, "table")
                .sync_create()
                .build();
            binding_group
                .bind("headers", &*self.header_pane, "table")
                .sync_create()
                .build();
            binding_group
                .bind("variables", &*self.variable_pane, "table")
                .sync_create()
                .build();
            binding_group
                .bind(
                    "authentication",
                    &*self.authentication_pane,
                    "authentication",
                )
                .sync_create()
                .build();
            binding_group
                .bind("body", &*self.body_pane, "body")
                .sync_create()
                .build();

            binding_group.set_source(Some(&self.obj().request()));

            let binding_group_2 = binding_group.clone();
            self.obj().connect_request_notify(glib::clone!(
                #[weak]
                binding_group_2,
                move |ep: &super::EndpointPane| {
                    binding_group_2.set_source(Some(&ep.request()));
                }
            ));
        }

        pub(super) fn get_response_object(&self) -> Option<Response> {
            self.response_pane.response()
        }

        fn has_response_impl(&self) -> bool {
            self.response_pane.response().is_some()
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
            let params_table = self.obj().request().params();
            let params = params_table
                .iter::<Field>()
                .filter_map(|p| p.ok())
                .filter(|field| field.active())
                .map(|field| (field.key(), field.value()))
                .collect::<Vec<(String, String)>>();
            let current_url = self.request_url.text().to_string();
            let next_url = super::update_queryparams(&current_url, &params);
            self.request_url.set_text(&next_url);
        }

        fn update_query_params(&self) {
            let url = self.request_url.text().to_string();
            let params = super::extract_queryparams(&url);
            self.obj().request().params().reconcile(&params);
        }

        fn init_dirty_events(&self) {
            let obj = self.obj();

            let request_signal_group = glib::SignalGroup::new::<Request>();
            request_signal_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &Request, _: &str| {
                        obj.set_dirty(true);
                    }
                ),
            );

            request_signal_group.set_target(Some(&obj.request()));
            obj.connect_request_notify(glib::clone!(
                #[weak]
                request_signal_group,
                move |pane| {
                    request_signal_group.set_target(Some(&pane.request()));
                }
            ));

            self.request_signal_group.set(request_signal_group).unwrap();
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
            // It is important to allow the redundant pattern matching because
            // is_ok() does not capture the mutex and will cause sync issues.
            #[allow(clippy::redundant_pattern_matching)]
            if let Ok(_) = self.variable_changing.try_lock() {
                self.update_query_params();
            }
        }

        #[template_callback]
        fn on_url_activated(&self) {
            let _ = self.obj().activate_action("endpoint.request", None);
        }

        #[template_callback]
        fn on_parameters_change(&self) {
            // It is important to allow the redundant pattern matching because
            // is_ok() does not capture the mutex and will cause sync issues.
            #[allow(clippy::redundant_pattern_matching)]
            if let Ok(_) = self.variable_changing.try_lock() {
                self.update_url_from_query_params();
            }
        }

        fn action_cancel_request(&self) {
            let obj = self.obj();

            {
                let maybe_request_thread = self.request_thread.borrow();
                if let Some(ref thread_ref) = *maybe_request_thread {
                    thread_ref.abort();

                    /* reset the user interface state. */
                    self.response_pane.set_spinning(false);
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
                    imp.response_pane.set_spinning(true);

                    imp.perform_request().await;

                    /* restore */
                    imp.response_pane.set_spinning(false);
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
            cartero_http::RequestEnvironment {
                config,
                prefix: self.obj().file().and_then(|f| f.parent()),
            }
        }

        /// Executes an HTTP request based on the current contents of the pane.
        pub(super) async fn perform_request(&self) {
            let request = self.obj().request();
            let env = self.request_environment();

            let response = match cartero_isahc_client::request(&request, &env).await {
                Err(RequestError::MissingProtocol) => {
                    let url_with_protocol = format!("http://{}", request.url());
                    self.obj().request().set_url(url_with_protocol);
                    let request = self.obj().request();
                    cartero_isahc_client::request(&request, &env).await
                }
                any => any,
            };

            match response {
                Ok(response) => {
                    self.response_pane.assign_from_response(&response);
                }
                Err(e) => {
                    self.response_pane.present_error(e);
                }
            };
        }
    }

    #[cfg(test)]
    mod tests {
        use cartero_objects::Field;
        use glib::subclass::types::ObjectSubclassIsExt;
        use gtk::prelude::EditableExt;
        use sourceview5::prelude::ListModelExt;

        use crate::app::CarteroApplication;

        use super::super::EndpointPane;

        fn assert_row(row: &Field, name: &str, value: &str, active: bool, secret: bool) {
            assert_eq!(row.key(), name);
            assert_eq!(row.value(), value);
            assert_eq!(row.active(), active);
            assert_eq!(row.masked(), secret);
        }

        #[gtk::test]
        fn test_setting_the_url_address_updates_params() {
            crate::init_test_resources();
            let _app = CarteroApplication::new();

            let pane = EndpointPane::default();
            let imp = pane.imp();

            /* So far, only the placeholder row in the param pane. */
            assert_eq!(0, imp.parameter_pane.table().n_items());

            imp.request_url
                .set_text("https://www.example.com/foobar.html?a=1&b=2&c=3&d=4");
            assert_eq!(4, imp.parameter_pane.table().n_items());
            assert_row(
                &imp.parameter_pane.table().field(0).unwrap(),
                "a",
                "1",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.table().field(1).unwrap(),
                "b",
                "2",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.table().field(2).unwrap(),
                "c",
                "3",
                true,
                false,
            );
            assert_row(
                &imp.parameter_pane.table().field(3).unwrap(),
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

            imp.parameter_pane.table().field(2).unwrap().set_value("9");
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

            imp.parameter_pane
                .table()
                .field(2)
                .unwrap()
                .set_active(false);
            assert_eq!(
                imp.request_url.text(),
                "https://www.example.com/foobar.html?a=1&b=2&d=4"
            );
        }
    }
}

glib::wrapper! {
    pub struct EndpointPane(ObjectSubclass<imp::EndpointPane>)
        @extends gtk::Widget, gtk::Box, BasePane,
        @implements gio::ActionMap;
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

    pub async fn export_response(&self) {
        let root = self.root().and_downcast::<gtk::Window>().unwrap();
        let export_file = file_dialogs::export_file(&root).await;
        match export_file {
            Err(e) => crate::widgets::dialogs::glib_file_dialog_error(&root, &e).await,
            Ok(file) => {
                if let Some(file) = file {
                    // get the current response payload
                    let resp = self.imp().get_response_object().unwrap();
                    let payload = resp.body().unwrap_or(glib::Bytes::from_static(&[]));
                    if let Err((_, e)) = file
                        .replace_contents_future(
                            payload.to_vec(),
                            None,
                            false,
                            FileCreateFlags::NONE,
                        )
                        .await
                    {
                        crate::widgets::dialogs::glib_file_dialog_error(&root, &e).await;
                    }
                }
            }
        }
    }

    pub async fn export_request(&self, format: &str) {
        let command = match format {
            "curl" => {
                let curl = CodeExportService::new(self.request());
                curl.generate().await
            }
            "jetbrains-http" => cartero_jetbrains_http_format::export(&self.request()).await,
            _ => {
                return;
            }
        };

        let file_format = match format {
            "curl" => sourceview5::LanguageManager::default().language("sh"),
            _ => None,
        };

        match command {
            Ok(command) => {
                let buffer = glib::Bytes::from(command.as_bytes());
                let dialog = glib::Object::builder::<ExportDialog>()
                    .property("blob", Some(&buffer))
                    .property("format", file_format)
                    .build();

                let title = match format {
                    "curl" => gettext("Export request as cURL"),
                    "jetbrains-http" => gettext("Export request as Jetbrains HTTP"),
                    _ => {
                        return;
                    }
                };
                dialog.set_title(&title);
                dialog.present(Some(self));
            }
            Err(e) => {
                println!("{:?}", e);
            }
        };
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
