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

use adw::prelude::*;
use adw::subclass::prelude::*;

glib::wrapper! {
    pub struct Proxy(ObjectSubclass<imp::Proxy>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Proxy {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use crate::settings::Settings;

    use super::*;

    use formatx::formatx;
    use gettextrs::gettext;
    use glib::subclass::InitializingObject;
    use gtk::{
        CompositeTemplate,
        gio::{SimpleAction, SimpleActionGroup},
    };

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/page_proxy.ui")]
    pub struct Proxy {
        settings: Settings,

        #[template_child]
        option_proxy_http: TemplateChild<adw::EntryRow>,
        #[template_child]
        option_proxy_https: TemplateChild<adw::EntryRow>,
        #[template_child]
        option_std_http: TemplateChild<adw::ActionRow>,
        #[template_child]
        option_std_https: TemplateChild<adw::ActionRow>,
        #[template_child]
        option_std_no_proxy: TemplateChild<adw::ActionRow>,
        #[template_child]
        add_excluded: TemplateChild<gtk::Entry>,
        #[template_child]
        add_error_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        add_error: TemplateChild<gtk::Label>,
        #[template_child]
        excluded_hosts_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Proxy {
        const NAME: &'static str = "CarteroSettingsPageProxy";
        type Type = super::Proxy;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Proxy {
        fn constructed(&self) {
            self.parent_constructed();

            self.render_hosts_list();
            self.settings.connect_changed(
                Some("proxy-no-proxy"),
                glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _| {
                        imp.render_hosts_list();
                    }
                ),
            );

            let action_add_excluded_host = SimpleAction::new("add-excluded-host", None);
            action_add_excluded_host.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.add_excluded_host();
                }
            ));
            self.add_excluded
                .property_expression("text")
                .chain_closure::<bool>(glib::closure_local!(|_: glib::Object, text: &str| {
                    !text.is_empty()
                }))
                .bind(
                    &action_add_excluded_host,
                    "enabled",
                    Some(&*self.add_excluded),
                );
            self.add_excluded.connect_activate(glib::clone!(
                #[weak]
                action_add_excluded_host,
                move |_| {
                    action_add_excluded_host.activate(None);
                }
            ));

            let action_remove_excluded_host =
                SimpleAction::new("remove-excluded-host", Some(&String::static_variant_type()));
            action_remove_excluded_host.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, value| {
                    if let Some(maybe_host) = value {
                        let host = maybe_host
                            .get::<String>()
                            .expect("Given host is not a string?");
                        glib::spawn_future_local(async move {
                            if imp.confirm_remove_host(&host).await {
                                imp.remove_excluded_host(&host);
                            }
                        });
                    }
                }
            ));

            let action_use_std = self.settings.create_action("proxy-use-env");
            let action_group = SimpleActionGroup::new();
            action_group.add_action(&action_use_std);
            action_group.add_action(&action_add_excluded_host);
            action_group.add_action(&action_remove_excluded_host);
            self.obj()
                .insert_action_group("widget", Some(&action_group));

            self.settings
                .bind("proxy-http", &*self.option_proxy_http, "text")
                .build();
            self.settings
                .bind("proxy-https", &*self.option_proxy_https, "text")
                .build();

            self.option_std_http.set_subtitle(&env_var("http_proxy"));
            self.option_std_https
                .set_subtitle(&env_var_or("https_proxy", "HTTPS_PROXY"));
            self.option_std_no_proxy
                .set_subtitle(&env_var_or("no_proxy", "NO_PROXY"));
        }
    }

    impl WidgetImpl for Proxy {}

    impl PreferencesPageImpl for Proxy {}

    impl Proxy {
        fn get_excluded_hosts(&self) -> Vec<String> {
            self.settings.get::<Vec<String>>("proxy-no-proxy")
        }

        fn set_excluded_hosts(&self, sites: &[String]) {
            if self.settings.set("proxy-no-proxy", sites).is_err() {
                glib::g_warning!("es.danirod.Cartero", "proxy-no-proxy update error");
            }
        }

        fn add_excluded_host(&self) {
            let host = self.add_excluded.text();
            // As a good will, remove any protocols present in the host.
            let clean_host = host
                .trim()
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim();
            // The host must be a valid one, without weird characters.
            let is_valid_input = match url::Host::parse(clean_host) {
                Ok(url::Host::Domain(d)) => addr::parse_domain_name(d.as_str()).is_ok(),
                Ok(url::Host::Ipv4(_) | url::Host::Ipv6(_)) => true,
                Err(_) => false,
            };
            if !is_valid_input {
                // Except that we should accept IPv6 addresses without brackets.
                let maybe_ipv6 = format!("[{}]", clean_host);
                if !matches!(url::Host::parse(&maybe_ipv6), Ok(url::Host::Ipv6(_))) {
                    self.add_error
                        .set_text(&gettext("The value is not a valid domain or IP address"));
                    self.add_error_revealer.set_reveal_child(true);
                    return;
                }
            }

            // Also, if the value is a valid IPv6 address, remove the brackets around it.
            let clean_host = match url::Host::parse(clean_host) {
                Ok(url::Host::Ipv6(_)) => clean_host.trim_start_matches("[").trim_end_matches("]"),
                _ => clean_host,
            };

            let mut sites = self.get_excluded_hosts();
            if sites.iter().any(|host| clean_host == host) {
                self.add_error
                    .set_text(&gettext("The host identifier is already excluded"));
                self.add_error_revealer.set_reveal_child(true);
                return;
            }

            sites.push(clean_host.to_string());
            self.set_excluded_hosts(&sites);

            self.add_error.set_text("");
            self.add_error_revealer.set_reveal_child(false);
            self.add_excluded.set_text("");
        }

        async fn confirm_remove_host(&self, host: &str) -> bool {
            let parent = self
                .obj()
                .root()
                .and_downcast::<gtk::Window>()
                .expect("Widget is not attached to a window");
            let body = formatx!(
                gettext("Delete '{}' from the list of excluded hosts for the proxy?"),
                host
            )
            .unwrap();
            let dialog = adw::AlertDialog::builder()
                .heading(gettext("Delete identifier?"))
                .body(&body)
                .build();
            dialog.add_response("cancel", &gettext("Cancel"));
            dialog.add_response("remove", &gettext("Remove"));
            dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
            let response = dialog.choose_future(Some(&parent)).await;
            "remove" == response
        }

        fn remove_excluded_host(&self, host: &str) {
            let mut sites = self.get_excluded_hosts();
            sites.retain(|s| s != host);
            self.set_excluded_hosts(&sites);
        }

        fn render_hosts_list(&self) {
            let sites = self.settings.get::<Vec<String>>("proxy-no-proxy");

            self.excluded_hosts_list.remove_all();
            if sites.is_empty() {
                let action_row = adw::ActionRow::builder()
                    .title(gettext("No domains or IP addresses are being excluded"))
                    .sensitive(false)
                    .halign(gtk::Align::Center)
                    .build();
                self.excluded_hosts_list.append(&action_row);
            } else {
                for host in sites {
                    let action_row = adw::ActionRow::builder()
                        .title(&host)
                        .selectable(false)
                        .title_selectable(true)
                        .build();
                    let remove_host = gtk::Button::builder()
                        .icon_name("user-trash-symbolic")
                        .action_name("widget.remove-excluded-host")
                        .action_target(&host.to_variant())
                        .tooltip_text(gettext("Remove"))
                        .css_classes(["destructive-action"])
                        .vexpand(false)
                        .valign(gtk::Align::Center)
                        .build();
                    action_row.add_suffix(&remove_host);
                    self.excluded_hosts_list.append(&action_row);
                }
            }
        }
    }

    fn env_var(key: &str) -> String {
        let var = std::env::var(key).unwrap_or_default();
        if var.is_empty() {
            gettext("(none)")
        } else {
            var
        }
    }

    fn env_var_or(key: &str, alt: &str) -> String {
        let var = std::env::var(key).unwrap_or_default();
        if var.is_empty() {
            let var = std::env::var(alt).unwrap_or_default();
            if var.is_empty() {
                gettext("(none)")
            } else {
                var
            }
        } else {
            var
        }
    }
}
