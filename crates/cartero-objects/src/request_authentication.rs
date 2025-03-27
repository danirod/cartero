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

use glib::subclass::prelude::*;
use glib::{prelude::*, Object};

use crate::{RequestAuthenticationBasic, RequestAuthenticationBearer};

glib::wrapper! {
    pub struct RequestAuthentication(ObjectSubclass<imp::RequestAuthentication>);
}

impl Default for RequestAuthentication {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl RequestAuthentication {
    pub fn new() -> Self {
        Object::builder()
            .property("auth-type", RequestAuthenticationType::None)
            .build()
    }

    pub fn basic_auth(&self) -> Option<RequestAuthenticationBasic> {
        if self.auth_type() == RequestAuthenticationType::BasicAuth {
            self.auth_data()
                .and_downcast::<RequestAuthenticationBasic>()
        } else {
            None
        }
    }

    pub fn bearer_token(&self) -> Option<RequestAuthenticationBearer> {
        if self.auth_type() == RequestAuthenticationType::BearerToken {
            self.auth_data()
                .and_downcast::<RequestAuthenticationBearer>()
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestAuthenticationType")]
pub enum RequestAuthenticationType {
    #[default]
    #[enum_value(name = "NONE", nick = "None")]
    None,
    #[enum_value(name = "INHERIT", nick = "Inherit")]
    Inherit,
    #[enum_value(name = "BASIC_AUTHENTICATION", nick = "Basic Authentication")]
    BasicAuth,
    #[enum_value(name = "BEARER_TOKEN", nick = "Bearer Token")]
    BearerToken,
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use crate::{
        RequestAuthenticationBasic, RequestAuthenticationBearer, RequestAuthenticationData,
    };

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestAuthentication)]
    pub struct RequestAuthentication {
        #[property(get, set, name = "auth-type", builder(RequestAuthenticationType::None))]
        auth_type: RefCell<RequestAuthenticationType>,

        #[property(get, name = "auth-data", nullable)]
        auth_data: RefCell<Option<RequestAuthenticationData>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestAuthentication {
        const NAME: &'static str = "CarteroRequestAuthorization";
        type Type = super::RequestAuthentication;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestAuthentication {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_auth_type_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |auth| {
                    let next = match auth.auth_type() {
                        RequestAuthenticationType::None => None,
                        RequestAuthenticationType::Inherit => None,
                        RequestAuthenticationType::BasicAuth => Some(
                            RequestAuthenticationBasic::default()
                                .upcast::<RequestAuthenticationData>(),
                        ),
                        RequestAuthenticationType::BearerToken => Some(
                            RequestAuthenticationBearer::default()
                                .upcast::<RequestAuthenticationData>(),
                        ),
                    };
                    imp.auth_data.replace(next);
                    auth.notify_auth_data();
                }
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test::assert_emits_signal;

    use super::*;

    #[test]
    pub fn no_authentication() {
        let authentication = RequestAuthentication::new();
        assert_emits_signal(&authentication, "notify::auth-data", || {
            authentication.set_auth_type(RequestAuthenticationType::None)
        });
        assert!(authentication.basic_auth().is_none());
        assert!(authentication.bearer_token().is_none());
    }

    #[test]
    pub fn inherit_authentication() {
        let authentication = RequestAuthentication::new();
        assert_emits_signal(&authentication, "notify::auth-data", || {
            authentication.set_auth_type(RequestAuthenticationType::Inherit)
        });
        assert!(authentication.basic_auth().is_none());
        assert!(authentication.bearer_token().is_none());
    }

    #[test]
    pub fn basic_auth() {
        let authentication = RequestAuthentication::new();
        assert_emits_signal(&authentication, "notify::auth-data", || {
            authentication.set_auth_type(RequestAuthenticationType::BasicAuth)
        });
        let basic_auth = authentication.basic_auth().unwrap();
        basic_auth.set_username("admin");
        basic_auth.set_password("1234");
        assert!(authentication
            .basic_auth()
            .is_some_and(|a| a.username() == "admin" && a.password() == "1234"));
        assert!(authentication.bearer_token().is_none());
    }

    #[test]
    pub fn bearer_auth() {
        let authentication = RequestAuthentication::new();
        assert_emits_signal(&authentication, "notify::auth-data", || {
            authentication.set_auth_type(RequestAuthenticationType::BearerToken)
        });
        let bearer_token = authentication.bearer_token().unwrap();
        bearer_token.set_token("55aa55aa");
        assert!(authentication.basic_auth().is_none());
        assert!(authentication
            .bearer_token()
            .is_some_and(|ba| ba.token() == "55aa55aa"));
    }
}
