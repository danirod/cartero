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

use glib::subclass::prelude::*;
use glib::{Object, prelude::*};

glib::wrapper! {
    /// Authentication type based on the RFC 7617 spec.
    ///
    /// In basic authentication, an username and a password are concatenated
    /// by a colon symbol, and then encoded as Base64. Then, the value is
    /// added to the `Authentication` HTTP header prefixed by the string
    /// `"Basic "`. For instance, to authorize as `root` with the password
    /// `toor`, the HTTP request requires a header called `Authorization` with
    /// the value set to `Basic cm9vdDp0b29y`. Read the RFC for more help.
    ///
    /// ## Properties
    ///
    /// - `username`: the username in use.
    /// - `password`: the password in use.
    ///
    /// ## Setting up an instance
    ///
    /// - Use the `default` function to define an empty auth-data object.
    /// - Use the [`new`][RequestAuthenticationBasic::new] function to assign
    ///   an initial username and password.
    pub struct RequestAuthenticationBasic(ObjectSubclass<imp::RequestAuthenticationBasic>)
        @extends crate::RequestAuthenticationData;
}

impl Default for RequestAuthenticationBasic {
    fn default() -> Self {
        Object::new()
    }
}

impl RequestAuthenticationBasic {
    /// Create a new authentication object with the given `username` and `password`.
    pub fn new(username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        Object::builder()
            .property("username", username.as_ref())
            .property("password", password.as_ref())
            .build()
    }

    pub fn builder() -> builder::RequestAuthenticationBasicBuilder {
        builder::RequestAuthenticationBasicBuilder::default()
    }
}

mod imp {
    use std::cell::RefCell;

    use base64::{Engine, prelude::BASE64_STANDARD};
    use glib::Properties;

    use crate::RequestAuthenticationDataImpl;

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestAuthenticationBasic)]
    pub struct RequestAuthenticationBasic {
        #[property(get, set)]
        username: RefCell<String>,

        #[property(get, set)]
        password: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestAuthenticationBasic {
        const NAME: &'static str = "CarteroRequestAuthorizationBasic";
        type Type = super::RequestAuthenticationBasic;
        type ParentType = crate::RequestAuthenticationData;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestAuthenticationBasic {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_username_notify(|auth| {
                auth.emit_by_name::<()>("changed", &[&"username"]);
            });
            self.obj().connect_password_notify(|auth| {
                auth.emit_by_name::<()>("changed", &[&"password"]);
            });
        }
    }

    impl RequestAuthenticationDataImpl for RequestAuthenticationBasic {
        fn dup(&self) -> crate::RequestAuthenticationData {
            super::RequestAuthenticationBasic::builder()
                .username(self.obj().username().to_string())
                .password(self.obj().password().to_string())
                .build()
                .upcast()
        }

        fn auth_type(&self) -> crate::RequestAuthenticationType {
            crate::RequestAuthenticationType::BasicAuth
        }

        fn resolve(
            &self,
            tpl: &srtemplate::SrTemplate,
        ) -> Result<crate::RequestAuthenticationData, srtemplate::Error> {
            let user = tpl.render(self.obj().username())?;
            let pass = tpl.render(self.obj().password())?;
            Ok(super::RequestAuthenticationBasic::builder()
                .username(user)
                .password(pass)
                .build()
                .upcast())
        }

        fn rendered_headers(&self) -> Vec<(String, String)> {
            let username = self.obj().username();
            let password = self.obj().password();
            let input = format!("{username}:{password}");
            let hash = BASE64_STANDARD.encode(input);
            let basic_auth_header = format!("Basic {hash}");
            vec![("Authorization".to_string(), basic_auth_header)]
        }
    }
}

mod builder {
    use glib::object::ObjectBuilder;

    use super::*;

    pub struct RequestAuthenticationBasicBuilder {
        builder: ObjectBuilder<'static, RequestAuthenticationBasic>,
    }

    impl Default for RequestAuthenticationBasicBuilder {
        fn default() -> Self {
            let builder = Object::builder();
            Self { builder }
        }
    }

    impl RequestAuthenticationBasicBuilder {
        pub fn build(self) -> RequestAuthenticationBasic {
            self.builder.build()
        }

        pub fn username<T>(mut self, username: T) -> Self
        where
            T: AsRef<str>,
        {
            self.builder = self.builder.property("username", username.as_ref());
            self
        }

        pub fn password<T>(mut self, password: T) -> Self
        where
            T: AsRef<str>,
        {
            self.builder = self.builder.property("password", password.as_ref());
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use srtemplate::SrTemplate;

    use crate::{
        RequestAuthenticationDataExt, RequestAuthenticationType, utils::test::assert_emits_signal,
    };

    use super::*;

    #[test]
    pub fn test_default() {
        let basic = RequestAuthenticationBasic::default();
        assert_eq!(basic.username(), String::default());
        assert_eq!(basic.password(), String::default());
    }

    #[test]
    pub fn test_new() {
        let basic = RequestAuthenticationBasic::new("admin", "1234");
        assert_eq!(basic.username(), "admin");
        assert_eq!(basic.password(), "1234");
    }

    #[test]
    pub fn test_auth_type() {
        let auth_type = RequestAuthenticationBasic::default().auth_type();
        assert_eq!(auth_type, RequestAuthenticationType::BasicAuth);
    }

    #[test]
    pub fn test_builder_default() {
        let basic = RequestAuthenticationBasic::builder().build();
        assert_eq!(basic.username(), String::default());
        assert_eq!(basic.password(), String::default());
        assert_eq!(basic.auth_type(), RequestAuthenticationType::BasicAuth);
    }

    #[test]
    pub fn test_builder_credentials() {
        let basic = RequestAuthenticationBasic::builder()
            .username("admin")
            .password("1234")
            .build();
        assert_eq!(basic.username(), "admin");
        assert_eq!(basic.password(), "1234");
        assert_eq!(basic.auth_type(), RequestAuthenticationType::BasicAuth);
    }

    #[test]
    pub fn test_emits_signals() {
        let basic = RequestAuthenticationBasic::default();
        assert_emits_signal(&basic, "changed", || basic.set_username("foo"));
        assert_emits_signal(&basic, "changed", || basic.set_password("bar"));
    }

    #[test]
    pub fn test_resolve_successful() {
        let auth = RequestAuthenticationBasic::builder()
            .username("{{USER}}")
            .password("{{PASS}}")
            .build();
        let tpl = SrTemplate::default();
        tpl.add_variable("USER", "admin");
        tpl.add_variable("PASS", "1234");
        let auth = auth
            .resolve(&tpl)
            .expect("Invalid resolve?")
            .downcast::<RequestAuthenticationBasic>()
            .expect("Invalid cast?");
        assert_eq!(auth.username(), "admin");
        assert_eq!(auth.password(), "1234");
    }

    #[test]
    #[should_panic]
    pub fn test_resolve_unsuccessful() {
        let auth = RequestAuthenticationBasic::builder()
            .username("{{USER}}")
            .password("{{PASS}}")
            .build();
        let tpl = SrTemplate::default();
        auth.resolve(&tpl).expect("Invalid resolve?");
    }

    #[test]
    pub fn test_rendered_headers() {
        let auth = RequestAuthenticationBasic::builder()
            .username("operator")
            .password("operator")
            .build();
        let headers = auth.rendered_headers();
        assert_eq!(1, headers.len());
        assert_eq!("Authorization", headers[0].0);
        assert_eq!("Basic b3BlcmF0b3I6b3BlcmF0b3I=", headers[0].1);
    }
}
