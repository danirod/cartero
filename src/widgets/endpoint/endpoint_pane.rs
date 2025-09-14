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

use glib::Object;
use gtk::gio;
use gtk::glib;
use url::form_urlencoded;

use crate::widgets::shell::BasePane;
mod imp {
    use std::cell::{OnceCell, RefCell};
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    use adw::prelude::AdwDialogExt;
    use adw::subclass::breakpoint_bin::BreakpointBinImpl;
    use cartero_http::RequestError;
    use cartero_isahc_client::default_user_agent;
    use cartero_objects::{
        EnvFile, Field, Request, RequestAuthenticationDataExt, RequestBodyDataExt, RequestBodyType,
        Response,
    };
    use formatx::formatx;
    use gettextrs::{gettext, ngettext};
    use glib::subclass::InitializingObject;
    use glib::{JoinHandle, Properties};
    use gtk::gio::{self, Cancellable, FileCreateFlags, SimpleAction, SimpleActionGroup};
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, ClosureExpression, CompositeTemplate};

    use crate::app::CarteroApplication;
    use crate::config::BASE_ID;
    use crate::interop::{InnerError, LoadResult, SaveResult};
    use crate::widgets::authentication::AuthenticationPane;
    use crate::widgets::dialogs::export_dialog_error;
    use crate::widgets::endpoint::ResponsePanel;
    use crate::widgets::field::{CollapsedFieldTable, FieldTableListView};
    use crate::widgets::req_body::RequestBodyPane;
    use crate::widgets::shell::BasePaneImpl;
    use crate::widgets::{file_dialogs, ExportDialog, MethodDropdown};

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/es/danirod/Cartero/endpoint_pane.ui")]
    #[properties(wrapper_type = super::EndpointPane)]
    pub struct EndpointPane {
        #[template_child]
        shortcuts: TemplateChild<gtk::ShortcutController>,
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
        #[template_child]
        pregenerated_headers: TemplateChild<CollapsedFieldTable>,
        #[template_child]
        env_variables: TemplateChild<CollapsedFieldTable>,
        #[template_child]
        env_file_status: TemplateChild<gtk::Stack>,

        #[property(get, set, name = "read-only")]
        read_only: RefCell<bool>,

        #[property(get, set)]
        request: RefCell<Request>,
        request_signal_group: OnceCell<glib::SignalGroup>,

        #[property(get, set)]
        show_pregenerated_headers: RefCell<bool>,
        #[property(get, set)]
        show_env_variables: RefCell<bool>,
        #[property(get, set)]
        env_file: RefCell<EnvFile>,

        #[property(get)]
        request_binding_group: RefCell<glib::BindingGroup>,

        #[property(get, set, nullable)]
        file: RefCell<Option<gio::File>>,

        #[property(get, set)]
        dirty: RefCell<bool>,

        // Busy requesting
        #[property(get, set)]
        busy: RefCell<bool>,

        request_thread: Arc<RefCell<Option<JoinHandle<()>>>>,
        action_group: OnceCell<gio::SimpleActionGroup>,
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
            self.init_shortcuts();

            self.init_request_binding_group();

            self.init_request_signal_group();
            self.init_pregenerated_rows();
            self.init_env_file();
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
        }
    }

    impl WidgetImpl for EndpointPane {}

    impl BreakpointBinImpl for EndpointPane {}

    impl BasePaneImpl for EndpointPane {
        fn lookup_action(&self, name: &str) -> Option<gio::Action> {
            self.action_group
                .get()
                .expect("No action_group is prepared")
                .lookup_action(name)
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
        fn init_env_file(&self) {
            self.update_env_file();
            self.obj().connect_file_notify(|pane| {
                pane.imp().update_env_file();
            });

            self.update_env_data();
            self.obj().env_file().connect_items_changed(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _, _, _| {
                    imp.update_env_data();
                }
            ));

            let app = CarteroApplication::get();
            let settings = app.settings();
            settings.connect_changed(
                Some("read-env-files"),
                glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _| {
                        imp.update_env_file();
                    }
                ),
            );
        }

        fn update_env_data(&self) {
            let entries = self
                .obj()
                .env_file()
                .iter::<Field>()
                .filter_map(|item| item.ok().map(|field| (field.key(), "*".repeat(4))))
                .collect::<Vec<(String, String)>>();
            let toggle_prompt = formatx!(
                ngettext(
                    "Show {} variable from .env",
                    "Show {} variables from .env",
                    entries.len() as u32
                ),
                entries.len()
            )
            .unwrap();
            let pregenerated = self.env_variables.field_table();
            pregenerated.reconcile(&entries);
            pregenerated.iter::<Field>().for_each(|item| {
                if let Ok(field) = item {
                    field.set_masked(true);
                }
            });
            self.env_variables.set_title(toggle_prompt.as_str());
            self.env_variables.set_subtitle(
                self.obj()
                    .env_file()
                    .file()
                    .map(|p| p.path().expect("No path for file?").display().to_string())
                    .unwrap_or_default()
                    .as_str(),
            );
        }

        fn update_env_file(&self) {
            let settings = gio::Settings::new(BASE_ID);
            let allow_env = settings.boolean("read-env-files");

            let env_file = if allow_env {
                self.obj()
                    .file()
                    .and_then(|file| EnvFile::locate_for_path(&file))
            } else {
                None
            };
            self.obj().env_file().set_file(env_file.as_ref());

            if allow_env {
                if self.obj().file().is_none() {
                    self.env_file_status.set_visible_child_name("unsaved-file");
                } else if env_file.is_none() {
                    self.env_file_status.set_visible_child_name("not-found");
                } else {
                    self.env_file_status.set_visible_child_name("toggle");
                }
            } else {
                self.env_file_status.set_visible_child_name("env-disabled");
            }
        }

        fn init_shortcuts(&self) {
            let focus_url_trigger = if cfg!(target_os = "macos") {
                "<Meta>l"
            } else {
                "<Primary>l"
            };
            let focus_url = gtk::Shortcut::builder()
                .trigger(&gtk::ShortcutTrigger::parse_string(focus_url_trigger).unwrap())
                .action(&gtk::ShortcutAction::parse_string("action(endpoint.focus-url)").unwrap())
                .build();
            self.shortcuts.add_shortcut(focus_url);
        }

        fn init_pregenerated_rows(&self) {
            self.update_pregenerated_headers();
            self.obj().connect_request_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_pregenerated_headers();
                }
            ));
        }

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

            let action_export_request =
                SimpleAction::new("export-request", Some(&String::static_variant_type()));
            action_export_request.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, variant| {
                    let format = variant
                        .expect("No variant is provided?")
                        .get::<String>()
                        .expect("No string variant is provided?");
                    glib::spawn_future_local(async move {
                        imp.action_export_request(format.as_ref()).await;
                    });
                }
            ));

            let action_export_response_body = SimpleAction::new("export-response-body", None);
            action_export_response_body.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    glib::spawn_future_local(async move {
                        imp.action_export_response().await;
                    });
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

            /* Response body can only be exported if there is a response at all. */
            self.response_pane
                .property_expression("response")
                .chain_closure::<bool>(glib::closure!(
                    move |_: glib::Object, response: Option<Response>| { response.is_some() }
                ))
                .bind(
                    &action_export_response_body,
                    "enabled",
                    Some(&*self.response_pane),
                );

            let action_reload_env = SimpleAction::new("reload-env", None);
            action_reload_env.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.update_env_file();
                }
            ));

            let action_group = SimpleActionGroup::new();
            action_group.add_action(&action_focus_url);
            action_group.add_action(&action_request);
            action_group.add_action(&action_cancel);
            action_group.add_action(&action_export_request);
            action_group.add_action(&action_export_response_body);
            action_group.add_action(&action_reload_env);
            obj.insert_action_group("endpoint", Some(&action_group));
            self.action_group.set(action_group).unwrap();
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

        fn update_pregenerated_headers(&self) {
            // Need a way to check which headers will be disabled.
            let user_headers = self
                .request
                .borrow()
                .headers()
                .iter::<Field>()
                .filter_map(|row| {
                    row.ok()
                        .take_if(|field| field.active())
                        .map(|field| field.key().trim().to_lowercase())
                })
                .collect::<HashSet<String>>();

            let pregenerated = self.pregenerated_headers.field_table();
            // TODO: These are dependant on the HTTP client, so they should be taken from there.
            let mut default_headers = vec![
                ("Accept".into(), "*/*".into()),
                ("Accept-Encoding".into(), "deflate, gzip".into()),
                ("Host".into(), gettext("(generated during request)")),
                ("User-Agent".into(), default_user_agent()),
            ];
            if self.obj().request().body().body_data().is_some() {
                default_headers.push((
                    "Content-Length".into(),
                    gettext("(generated during request)"),
                ));
            }
            let auth_headers = self
                .obj()
                .request()
                .authentication()
                .auth_data()
                .map(|auth| auth.rendered_headers())
                .unwrap_or_default();
            let body_headers = self
                .obj()
                .request()
                .body()
                .body_data()
                .map(|body| {
                    let mut headers = body.rendered_headers();
                    if body.body_type() == RequestBodyType::Multipart {
                        if let Some((_, value)) =
                            headers.iter_mut().find(|(key, _)| key == "Content-Type")
                        {
                            *value = format!("{}{}", *value, gettext("(generated during request)"));
                        }
                    }
                    headers
                })
                .unwrap_or_default();
            let mut entries = default_headers
                .into_iter()
                .chain(auth_headers)
                .chain(body_headers)
                .collect::<Vec<(String, String)>>();
            entries.sort_by_key(|(key, _)| key.clone());

            pregenerated.iter::<Field>().for_each(|row| {
                if let Ok(field) = row {
                    field.set_active(true);
                }
            });
            pregenerated.reconcile(&entries);
            pregenerated.iter::<Field>().for_each(|row| {
                if let Ok(field) = row {
                    let current_header = field.key().trim().to_lowercase();
                    let set_by_user = user_headers.contains(&current_header);
                    field.set_active(!set_by_user);
                }
            });

            let toggle_prompt =
                formatx!(gettext("Show {} pre-generated headers"), entries.len()).unwrap();
            self.pregenerated_headers.set_title(toggle_prompt.as_str());
        }

        fn init_request_signal_group(&self) {
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

            request_signal_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_: &Request, _: &str| {
                        imp.update_pregenerated_headers();
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
            let proxy = cartero_http::ProxyConfig {
                respect_system_proxy: settings.boolean("proxy-use-env"),
                http_proxy: settings.string("proxy-http").to_string(),
                https_proxy: settings.string("proxy-https").to_string(),
                no_proxy: settings
                    .strv("proxy-no-proxy")
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>(),
            };
            cartero_http::RequestEnvironment {
                config,
                env_file: Some(self.obj().env_file()),
                proxy: Some(proxy),
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

        async fn action_export_request(&self, format: &str) {
            let root = self.obj().root().and_downcast::<gtk::Window>().unwrap();
            let request = self.obj().request();

            let template = match format {
                "curl" => cartero_code_exporters::Format::Curl,
                "jetbrains-http" => cartero_code_exporters::Format::Ijhttp,
                _ => {
                    return;
                }
            };

            let encoded = match cartero_code_exporters::export_request(template, &request) {
                Ok(command) => command,
                Err(e) => {
                    export_dialog_error(&root, e).await;
                    return;
                }
            };

            let file_format = match format {
                "curl" => sourceview5::LanguageManager::default().language("sh"),
                _ => None,
            };

            let buffer = glib::Bytes::from(encoded.as_bytes());
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
            dialog.present(Some(&*self.obj()));
        }

        async fn action_export_response(&self) {
            let root = self.obj().root().and_downcast::<gtk::Window>().unwrap();
            let initial_file = self
                .response_pane
                .response()
                .expect("No response to export?")
                .file_name();
            let export_file = file_dialogs::export_file(&root, Some(initial_file.as_str())).await;
            match export_file {
                Err(e) => crate::widgets::dialogs::glib_file_dialog_error(&root, &e).await,
                Ok(file) => {
                    if let Some(file) = file {
                        // get the current response payload
                        let resp = self
                            .response_pane
                            .response()
                            .expect("No response to export?");
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
        @extends gtk::Widget, adw::BreakpointBin, BasePane,
        @implements gtk::Accessible, gio::ActionMap, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EndpointPane {
    fn default() -> Self {
        Object::builder().build()
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
