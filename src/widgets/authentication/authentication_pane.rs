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

use adw::prelude::*;
use adw::subclass::prelude::*;

mod imp {
    use crate::widgets::authentication::{BasicAuth, BearerToken};

    use super::*;
    use std::cell::RefCell;

    use cartero_objects::{
        RequestAuthentication, RequestAuthenticationBasic, RequestAuthenticationBearer,
        RequestAuthenticationType,
    };
    use glib::{subclass::InitializingObject, Object, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::AuthenticationPane)]
    #[template(resource = "/es/danirod/Cartero/authentication_pane.ui")]
    pub struct AuthenticationPane {
        #[property(get, set)]
        authentication: RefCell<RequestAuthentication>,
        #[property(get, set)]
        read_only: RefCell<bool>,

        last_basic_auth: RefCell<RequestAuthenticationBasic>,
        last_bearer_token: RefCell<RequestAuthenticationBearer>,

        #[template_child]
        container: TemplateChild<adw::Bin>,
        #[template_child]
        combo: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AuthenticationPane {
        const NAME: &'static str = "CarteroAuthenticationPane";
        type Type = super::AuthenticationPane;
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
    impl ObjectImpl for AuthenticationPane {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_authentication_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |pane: &super::AuthenticationPane| {
                    // This will eventually call the notify::selected signal of the combo.
                    let auth_type = pane.authentication().auth_type();
                    imp.combo.set_selected(cast_entry_to_select(auth_type));
                }
            ));
        }
    }

    impl WidgetImpl for AuthenticationPane {}

    impl BoxImpl for AuthenticationPane {}

    #[gtk::template_callbacks]
    impl AuthenticationPane {
        fn sync_container(&self) {
            let authentication = self.obj().authentication();
            let container_child = match authentication.auth_type() {
                RequestAuthenticationType::BasicAuth => {
                    let basic: BasicAuth = Object::builder()
                        .property("basic-auth", authentication.basic_auth().unwrap())
                        .build();
                    Some(basic.upcast::<gtk::Widget>())
                }
                RequestAuthenticationType::BearerToken => {
                    let bearer: BearerToken = Object::builder()
                        .property("bearer-token", authentication.bearer_token().unwrap())
                        .build();
                    Some(bearer.upcast::<gtk::Widget>())
                }
                _ => None,
            };
            self.container.set_child(container_child.as_ref());
        }

        fn push_authentication(&self) {
            let authentication = self.obj().authentication();
            match authentication.auth_type() {
                RequestAuthenticationType::BasicAuth => {
                    self.last_basic_auth
                        .replace(authentication.basic_auth().expect("Expected BasicAuth?"));
                }
                RequestAuthenticationType::BearerToken => {
                    self.last_bearer_token.replace(
                        authentication
                            .bearer_token()
                            .expect("Expected BearerToken?"),
                    );
                }
                _ => {}
            }
        }

        fn pop_authentication(&self) {
            let authentication = self.obj().authentication();
            match authentication.auth_type() {
                RequestAuthenticationType::BasicAuth => {
                    let data = self.last_basic_auth.borrow().clone();
                    authentication.set_auth_data(data.into());
                }
                RequestAuthenticationType::BearerToken => {
                    let data = self.last_bearer_token.borrow().clone();
                    authentication.set_auth_data(data.into());
                }
                _ => {}
            }
        }

        #[template_callback]
        fn on_selection_changed(&self) {
            self.push_authentication();
            let authentication = self.obj().authentication();
            authentication.set_auth_type(cast_selected_entry(self.combo.selected()));
            self.pop_authentication();
            self.sync_container();
        }
    }

    fn cast_selected_entry(value: u32) -> RequestAuthenticationType {
        match value {
            1 => RequestAuthenticationType::BasicAuth,
            2 => RequestAuthenticationType::BearerToken,
            _ => RequestAuthenticationType::None,
        }
    }

    fn cast_entry_to_select(auth_type: RequestAuthenticationType) -> u32 {
        match auth_type {
            RequestAuthenticationType::BasicAuth => 1,
            RequestAuthenticationType::BearerToken => 2,
            _ => 0,
        }
    }
}

glib::wrapper! {
    pub struct AuthenticationPane(ObjectSubclass<imp::AuthenticationPane>)
        @extends gtk::Widget, gtk::Box;
}
