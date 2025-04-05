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
    impl ObjectImpl for RequestAuthenticationBearer {}

    impl RequestAuthenticationDataImpl for RequestAuthenticationBearer {
        fn auth_type(&self) -> crate::RequestAuthenticationType {
            crate::RequestAuthenticationType::BearerToken
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{RequestAuthenticationDataExt, RequestAuthenticationType};

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
}
