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

use std::sync::OnceLock;

use glib::{object::ObjectExt, subclass::types::ObjectSubclassIsExt};
use gtk::glib;

use crate::entities::RequestAuthorization;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use adw::prelude::ComboRowExt;
    use glib::subclass::{InitializingObject, Signal};
    use glib::Properties;
    use gtk::glib;
    use gtk::prelude::{EditableExt, ObjectExt};
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use crate::entities::RequestAuthorization;

    use super::AuthorizationMethod;

    #[derive(Default, CompositeTemplate, Properties)]
    #[template(resource = "/es/danirod/Cartero/authorization_pane.ui")]
    #[properties(wrapper_type = super::AuthorizationPane)]
    pub struct AuthorizationPane {
        #[template_child]
        combo: TemplateChild<adw::ComboRow>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        basic_username: TemplateChild<adw::EntryRow>,

        #[template_child]
        basic_password: TemplateChild<adw::PasswordEntryRow>,

        #[template_child]
        bearer_token: TemplateChild<adw::PasswordEntryRow>,

        #[property(get, set, name = "read-only")]
        read_only: RefCell<bool>,

        #[property(
            get,
            set,
            name = "auth-method",
            builder(AuthorizationMethod::default())
        )]
        auth_method: RefCell<AuthorizationMethod>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AuthorizationPane {
        const NAME: &'static str = "CarteroAuthorizationPane";
        type Type = super::AuthorizationPane;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AuthorizationPane {
        fn constructed(&self) {
            self.parent_constructed();

            self.init_events();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("changed").build()])
        }
    }

    impl WidgetImpl for AuthorizationPane {}

    impl BoxImpl for AuthorizationPane {}

    #[gtk::template_callbacks]
    impl AuthorizationPane {
        fn init_events(&self) {
            let obj = self.obj();
            obj.connect_auth_method_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |pane: &super::AuthorizationPane| {
                    let method = pane.auth_method();
                    imp.on_auth_method_changed(method);
                    pane.emit_by_name::<()>("changed", &[]);
                }
            ));

            self.combo.connect_selected_notify(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }
            ));

            self.basic_username.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }
            ));

            self.basic_password.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }
            ));

            self.bearer_token.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }
            ));
        }

        fn current_dropdown_method(&self) -> AuthorizationMethod {
            let n_item = self.combo.selected();
            AuthorizationMethod::types()[n_item as usize]
        }

        fn on_auth_method_changed(&self, method: AuthorizationMethod) {
            // update the value presented in the combobox if needed
            if self.current_dropdown_method() != method {
                let pos = AuthorizationMethod::types()
                    .iter()
                    .position(|&t| t == method)
                    .unwrap();
                self.combo.set_selected(pos as u32);
            }

            // update the currently visible pane if needed
            let new_stack_child = match method {
                AuthorizationMethod::None => "none",
                AuthorizationMethod::Bearer => "bearer",
                AuthorizationMethod::Basic => "basic",
            };
            if !self
                .stack
                .visible_child_name()
                .is_some_and(|m| m == new_stack_child)
            {
                self.stack.set_visible_child_name(new_stack_child);
            }
        }

        // dropdown -> prop
        #[template_callback]
        fn on_selection_changed(&self) {
            let obj = self.obj();
            let method = self.current_dropdown_method();
            if obj.auth_method() != method {
                obj.set_auth_method(method);
            }
        }

        pub fn assign_authorization(&self, auth: &RequestAuthorization) {
            let obj = self.obj();
            match auth {
                RequestAuthorization::None => {
                    obj.set_auth_method(AuthorizationMethod::None);
                }
                RequestAuthorization::Basic { username, password } => {
                    obj.set_auth_method(AuthorizationMethod::Basic);
                    self.basic_username.set_text(&username);
                    self.basic_password.set_text(&password);
                }
                RequestAuthorization::Bearer(token) => {
                    obj.set_auth_method(AuthorizationMethod::Bearer);
                    self.bearer_token.set_text(&token);
                }
            }
        }

        pub fn extract_authorization(&self) -> RequestAuthorization {
            let obj = self.obj();
            match obj.auth_method() {
                AuthorizationMethod::None => RequestAuthorization::None,
                AuthorizationMethod::Basic => {
                    let username = self.basic_username.text().to_string();
                    let password = self.basic_password.text().to_string();
                    RequestAuthorization::Basic { username, password }
                }
                AuthorizationMethod::Bearer => {
                    let token = self.bearer_token.text().to_string();
                    RequestAuthorization::Bearer(token)
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct AuthorizationPane(ObjectSubclass<imp::AuthorizationPane>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

impl AuthorizationPane {
    pub fn connect_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "changed",
            true,
            glib::closure_local!(|ref pane| {
                f(pane);
            }),
        )
    }

    pub fn set_authorization(&self, auth: &RequestAuthorization) {
        let imp = self.imp();
        imp.assign_authorization(auth)
    }

    pub fn authorization(&self) -> RequestAuthorization {
        let imp = self.imp();
        imp.extract_authorization()
    }
}
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroAuthorizationMethod")]
pub enum AuthorizationMethod {
    #[default]
    None,
    Basic,
    Bearer,
}

impl AuthorizationMethod {
    pub fn types() -> &'static [Self] {
        static TYPES: OnceLock<Vec<AuthorizationMethod>> = OnceLock::new();
        TYPES.get_or_init(|| vec![Self::None, Self::Basic, Self::Bearer])
    }
}
