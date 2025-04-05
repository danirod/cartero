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

use crate::{RequestAuthenticationBasic, RequestAuthenticationBearer, RequestAuthenticationData};

fn default_authentication_data(
    auth_type: RequestAuthenticationType,
) -> Option<RequestAuthenticationData> {
    match auth_type {
        RequestAuthenticationType::BasicAuth => {
            Some(RequestAuthenticationBasic::default().upcast())
        }
        RequestAuthenticationType::BearerToken => {
            Some(RequestAuthenticationBearer::default().upcast())
        }
        _ => None,
    }
}

glib::wrapper! {
    pub struct RequestAuthentication(ObjectSubclass<imp::RequestAuthentication>);
}

impl Default for RequestAuthentication {
    fn default() -> Self {
        Object::new()
    }
}

impl RequestAuthentication {
    pub fn new<T>(auth_type: RequestAuthenticationType, auth_data: Option<T>) -> Self
    where
        T: IsA<RequestAuthenticationData>,
    {
        let auth_data: Option<RequestAuthenticationData> = auth_data
            .map(|data| data.upcast())
            .or_else(|| default_authentication_data(auth_type));
        Object::builder()
            .property("auth-type", auth_type)
            .property("auth-data", auth_data)
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

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, glib::Enum)]
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

    use crate::{RequestAuthenticationData, RequestAuthenticationDataExt};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestAuthentication)]
    pub struct RequestAuthentication {
        #[property(get, set = Self::set_auth_type, name = "auth-type", builder(RequestAuthenticationType::None))]
        auth_type: RefCell<RequestAuthenticationType>,

        #[property(get, set = Self::set_auth_data, explicit_notify, name = "auth-data", nullable)]
        auth_data: RefCell<Option<RequestAuthenticationData>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestAuthentication {
        const NAME: &'static str = "CarteroRequestAuthorization";
        type Type = super::RequestAuthentication;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestAuthentication {}

    impl RequestAuthentication {
        // This is the inner setter for the auth-type property. It also changes the auth-data
        // to a new object of the appropiate type. The old contents of the auth-data are erased
        // in the process.
        fn set_auth_type(&self, auth_type: RequestAuthenticationType) {
            let next = default_authentication_data(auth_type);
            self.auth_type.replace(auth_type);
            self.obj().set_auth_data(next);
            self.obj().notify_auth_data();
        }

        // This is the inner setter for the auth-data property, which also verifies that the
        // type of the given data is acceptable for the current auth-type the object is set to.
        fn set_auth_data(&self, auth_data: Option<RequestAuthenticationData>) {
            let current_type = self.obj().auth_type();
            let valid = match current_type {
                RequestAuthenticationType::BasicAuth => auth_data
                    .as_ref()
                    .is_some_and(|data| data.auth_type() == current_type),
                RequestAuthenticationType::BearerToken => auth_data
                    .as_ref()
                    .is_some_and(|data| data.auth_type() == current_type),
                _ => auth_data.is_none(),
            };
            if valid {
                self.auth_data.replace(auth_data);
                self.obj().notify_auth_data();
            } else {
                #[cfg(not(test))]
                glib::g_critical!("Cartero", "set_auth_data() was called with a RequestAuthenticationData of invalid RequestAuthenticationType for this RequestAuthentication object");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::test::{assert_emits_signal, assert_emits_signals, assert_not_emits_signal},
        RequestAuthenticationDataExt,
    };

    use super::*;

    #[test]
    pub fn new_for_none() {
        let authentication = RequestAuthentication::new(
            RequestAuthenticationType::None,
            RequestAuthenticationData::NONE,
        );
        assert_eq!(RequestAuthenticationType::None, authentication.auth_type());
        assert!(authentication.auth_data().is_none());
    }

    #[test]
    pub fn new_for_inherit() {
        let authentication = RequestAuthentication::new(
            RequestAuthenticationType::Inherit,
            RequestAuthenticationData::NONE,
        );
        assert_eq!(
            RequestAuthenticationType::Inherit,
            authentication.auth_type()
        );
        assert!(authentication.auth_data().is_none());
    }

    #[test]
    pub fn new_for_basic_with_default() {
        let authentication = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            RequestAuthenticationData::NONE,
        );
        assert_eq!(
            RequestAuthenticationType::BasicAuth,
            authentication.auth_type()
        );
        let auth_data = authentication.auth_data().unwrap();
        assert_eq!(RequestAuthenticationType::BasicAuth, auth_data.auth_type());
        let basic_auth_data = auth_data.downcast::<RequestAuthenticationBasic>().unwrap();
        assert_eq!(basic_auth_data.username(), "");
        assert_eq!(basic_auth_data.password(), "");
    }

    #[test]
    pub fn new_for_basic_with_initial() {
        let credentials = RequestAuthenticationBasic::new("admin", "1234");
        let authentication =
            RequestAuthentication::new(RequestAuthenticationType::BasicAuth, Some(credentials));
        assert_eq!(
            RequestAuthenticationType::BasicAuth,
            authentication.auth_type()
        );
        let auth_data = authentication.auth_data().unwrap();
        assert_eq!(RequestAuthenticationType::BasicAuth, auth_data.auth_type());
        let basic_auth_data = auth_data.downcast::<RequestAuthenticationBasic>().unwrap();
        assert_eq!(basic_auth_data.username(), "admin");
        assert_eq!(basic_auth_data.password(), "1234");
    }

    #[test]
    pub fn new_for_bearer_with_default() {
        let authentication = RequestAuthentication::new(
            RequestAuthenticationType::BearerToken,
            RequestAuthenticationData::NONE,
        );
        assert_eq!(
            RequestAuthenticationType::BearerToken,
            authentication.auth_type()
        );
        let auth_data = authentication.auth_data().unwrap();
        assert_eq!(
            RequestAuthenticationType::BearerToken,
            auth_data.auth_type()
        );
        let bearer_auth_data = auth_data.downcast::<RequestAuthenticationBearer>().unwrap();
        assert_eq!(bearer_auth_data.token(), "");
    }

    #[test]
    pub fn new_for_bearer_with_initial() {
        let credentials = RequestAuthenticationBearer::new("auth_token");
        let authentication =
            RequestAuthentication::new(RequestAuthenticationType::BearerToken, Some(credentials));
        assert_eq!(
            RequestAuthenticationType::BearerToken,
            authentication.auth_type()
        );
        let auth_data = authentication.auth_data().unwrap();
        assert_eq!(
            RequestAuthenticationType::BearerToken,
            auth_data.auth_type()
        );
        let bearer_auth_data = auth_data.downcast::<RequestAuthenticationBearer>().unwrap();
        assert_eq!(bearer_auth_data.token(), "auth_token");
    }

    #[test]
    pub fn set_auth_type_changes_data_type() {
        let authentication = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            RequestAuthenticationData::NONE,
        );
        assert!(authentication
            .auth_data()
            .is_some_and(|data| data.auth_type() == RequestAuthenticationType::BasicAuth));
        assert_emits_signals(
            &authentication,
            &["notify::auth-type", "notify::auth-data"],
            || {
                authentication.set_auth_type(RequestAuthenticationType::BearerToken);
            },
        );
        assert!(authentication
            .auth_data()
            .is_some_and(|data| data.auth_type() == RequestAuthenticationType::BearerToken));
    }

    #[test]
    pub fn set_auth_data_with_same_type() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            RequestAuthenticationData::NONE,
        );
        let new_auth = RequestAuthenticationBasic::new("root", "toor");
        assert_emits_signal(&auth, "notify::auth-data", || {
            auth.set_auth_data(Some(new_auth.as_ref()))
        });
        let auth_data = auth
            .auth_data()
            .and_downcast::<RequestAuthenticationBasic>()
            .unwrap();
        assert_eq!(auth_data.username(), "root");
        assert_eq!(auth_data.password(), "toor");
    }

    #[test]
    pub fn set_auth_data_with_distinct_type() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            RequestAuthenticationData::NONE,
        );
        let new_auth = RequestAuthenticationBearer::new("token");
        assert_not_emits_signal(&auth, "notify::auth-data", || {
            auth.set_auth_data(Some(new_auth.as_ref()))
        });
    }

    #[test]
    pub fn basic_auth_when_basic_auth() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            RequestAuthenticationData::NONE,
        );
        assert!(auth.basic_auth().is_some());
    }

    #[test]
    pub fn basic_auth_when_not_basic_auth() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BearerToken,
            RequestAuthenticationData::NONE,
        );
        assert!(auth.basic_auth().is_none());
    }

    #[test]
    pub fn bearer_auth_when_bearer_auth() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BearerToken,
            RequestAuthenticationData::NONE,
        );
        assert!(auth.bearer_token().is_some());
    }

    #[test]
    pub fn bearer_auth_when_not_bearer_auth() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            RequestAuthenticationData::NONE,
        );
        assert!(auth.bearer_token().is_none());
    }
}
