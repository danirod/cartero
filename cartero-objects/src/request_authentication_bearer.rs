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

glib::wrapper! {
    /// Authentication type based on the RFC 6750 spec.
    ///
    /// In bearer authentication, a special token called bearer token is
    /// provided. This token is like a special and longer passcode issued by
    /// some authentication agent after checking the identity of the client,
    /// usually with a login protocol such as OAuth 2.0. The bearer token is
    /// provided as part of an HTTP request by adding an `Authorization` HTTP
    /// header with the value `Bearer`, followed by a space, and then the
    /// bearer token. For instance, `Bearer 0489161709`. Read the RFC for
    /// more help.
    ///
    /// ## Properties
    ///
    /// - `token`: the current token.
    ///
    /// ## Setting up an instance
    ///
    /// - Use the `default` function to define an empty auth-data object.
    /// - Use the [`new`][RequestAuthenticationBearer::new] function to assign
    ///   an initial token.
    pub struct RequestAuthenticationBearer(ObjectSubclass<imp::RequestAuthenticationBearer>)
        @extends crate::RequestAuthenticationData;
}

impl Default for RequestAuthenticationBearer {
    fn default() -> Self {
        Object::new()
    }
}

impl RequestAuthenticationBearer {
    pub fn new(token: impl AsRef<str>) -> Self {
        Object::builder().property("token", token.as_ref()).build()
    }

    pub fn builder() -> builder::RequestAuthenticationBearerBuilder {
        builder::RequestAuthenticationBearerBuilder::default()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use crate::RequestAuthenticationDataImpl;

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestAuthenticationBearer)]
    pub struct RequestAuthenticationBearer {
        #[property(get, set)]
        token: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestAuthenticationBearer {
        const NAME: &'static str = "CarteroRequestAuthorizationBearer";
        type Type = super::RequestAuthenticationBearer;
        type ParentType = crate::RequestAuthenticationData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestAuthenticationBearer {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_token_notify(|auth| {
                auth.emit_by_name::<()>("changed", &[&"token"]);
            });
        }
    }

    impl RequestAuthenticationDataImpl for RequestAuthenticationBearer {
        fn auth_type(&self) -> crate::RequestAuthenticationType {
            crate::RequestAuthenticationType::BearerToken
        }
    }
}

mod builder {
    use glib::object::ObjectBuilder;

    use super::*;

    pub struct RequestAuthenticationBearerBuilder {
        builder: ObjectBuilder<'static, RequestAuthenticationBearer>,
    }

    impl Default for RequestAuthenticationBearerBuilder {
        fn default() -> Self {
            let builder = Object::builder();
            Self { builder }
        }
    }

    impl RequestAuthenticationBearerBuilder {
        pub fn build(self) -> RequestAuthenticationBearer {
            self.builder.build()
        }

        pub fn token<T>(mut self, token: T) -> Self
        where
            T: AsRef<str>,
        {
            self.builder = self.builder.property("token", token.as_ref());
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::test::assert_emits_signal, RequestAuthenticationDataExt, RequestAuthenticationType,
    };

    use super::*;

    #[test]
    pub fn test_default() {
        let bearer = RequestAuthenticationBearer::default();
        assert_eq!(bearer.token(), String::default());
    }

    #[test]
    pub fn test_new() {
        let bearer = RequestAuthenticationBearer::new("auth_token");
        assert_eq!(bearer.token(), "auth_token");
    }

    #[test]
    pub fn test_auth_type() {
        let auth_type = RequestAuthenticationBearer::default().auth_type();
        assert_eq!(auth_type, RequestAuthenticationType::BearerToken);
    }

    #[test]
    pub fn test_builder_default() {
        let bearer = RequestAuthenticationBearer::builder().build();
        assert_eq!(bearer.token(), String::default());
        assert_eq!(bearer.auth_type(), RequestAuthenticationType::BearerToken);
    }

    #[test]
    pub fn test_builder_with_token() {
        let bearer = RequestAuthenticationBearer::builder()
            .token("aabbccdd")
            .build();
        assert_eq!(bearer.token(), "aabbccdd");
        assert_eq!(bearer.auth_type(), RequestAuthenticationType::BearerToken);
    }

    #[test]
    pub fn test_emits_signals() {
        let bearer = RequestAuthenticationBearer::default();
        assert_emits_signal(&bearer, "changed", || bearer.set_token("1234"));
    }
}
