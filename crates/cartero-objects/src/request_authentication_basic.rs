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
    pub struct RequestAuthenticationBasic(ObjectSubclass<imp::RequestAuthenticationBasic>)
        @extends crate::RequestAuthenticationData;
}

impl Default for RequestAuthenticationBasic {
    fn default() -> Self {
        Object::new()
    }
}

impl RequestAuthenticationBasic {
    pub fn new(username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        Object::builder()
            .property("username", username.as_ref())
            .property("password", password.as_ref())
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

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
    impl ObjectImpl for RequestAuthenticationBasic {}

    impl RequestAuthenticationDataImpl for RequestAuthenticationBasic {
        fn auth_type(&self) -> crate::RequestAuthenticationType {
            crate::RequestAuthenticationType::BasicAuth
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{RequestAuthenticationDataExt, RequestAuthenticationType};

    use super::RequestAuthenticationBasic;

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
}
