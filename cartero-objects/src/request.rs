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

use crate::RequestMethod;

glib::wrapper! {
    /// The high order class that represents a request.
    ///
    /// A `Request` class is made of the different information components that
    /// are needed in order to fully craft an HTTP request. These elements are
    /// meant to be presented to the user via the user interface to let the
    /// user get or change the values.
    ///
    /// ## Properties
    ///
    /// - `authentication`: a [RequestAuthentication][super::RequestAuthentication]
    ///   object to interact with the authentication data. This is later treated
    ///   as the `Authorization` header when sending a request.
    /// - `body`: a [RequestBody][super::RequestBody] object to interact with
    ///   the payload that some HTTP requests can carry when being performed.
    /// - `headers`: a [FieldTable][super::FieldTable] to collect the headers
    ///   to be added to a request.
    /// - `method`: a [RequestMethod][super::RequestMethod] enum value used
    ///   to indicate the verb.
    /// - `params`: a [FieldTable][super::FieldTable] that collects additional
    ///   query parameters. These are added to the URL during a request as
    ///   long as the field is enabled.
    /// - `url`: a String with the target URL where the request is pointing to.
    /// - `variables`: a [FieldTable][super::FieldTable] with variables that
    ///   are interpolated before sending an HTTP request, in order to un-hardcode
    ///   common things such as API tokens, passwords, roots...
    ///
    /// ## URL vs Params
    ///
    /// Both fields contradict themselves. The URL is a String that may carry
    /// a query string (the `?` character followed by zero, one or more
    /// urlencoded key-value pairs). The params table may also carry extra
    /// fields.
    ///
    /// It's not up to this crate to decide which one to pick. The values may
    /// be concatted, the table may carry only disabled parameters, or the URL
    /// may be stripped of the querystring and every parameter may be added
    /// into the table. But this is a task for caller code (such as the user
    /// interface or the file serialization API).
    pub struct Request(ObjectSubclass<imp::Request>);
}

impl Default for Request {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl Request {
    pub fn builder(url: &str, method: RequestMethod) -> builder::RequestBuilder {
        builder::RequestBuilder::new(url, method)
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use crate::{field_table::FieldTable, RequestAuthentication, RequestBody, RequestMethod};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Request)]
    pub struct Request {
        #[property(get, set, builder(RequestMethod::default()))]
        method: RefCell<RequestMethod>,

        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set)]
        params: RefCell<FieldTable>,

        #[property(get, set)]
        headers: RefCell<FieldTable>,

        #[property(get, set)]
        variables: RefCell<FieldTable>,

        #[property(get)]
        authentication: RefCell<RequestAuthentication>,

        #[property(get)]
        body: RefCell<RequestBody>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Request {
        const NAME: &'static str = "CarteroRequest";
        type Type = super::Request;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Request {}
}

mod builder {
    use crate::{RequestAuthentication, RequestBody};

    use super::*;
    use glib::object::ObjectBuilder;

    pub struct RequestBuilder {
        builder: ObjectBuilder<'static, Request>,
        authentication: RequestAuthentication,
        body: RequestBody,
    }

    impl RequestBuilder {
        pub fn new(url: &str, method: RequestMethod) -> Self {
            let builder = glib::Object::builder()
                .property("url", url)
                .property("method", method);
            let authentication = RequestAuthentication::default();
            let body = RequestBody::default();
            Self {
                builder,
                authentication,
                body,
            }
        }

        pub fn with_auth(mut self, authentication: RequestAuthentication) -> Self {
            self.authentication = authentication;
            self
        }

        pub fn with_body(mut self, body: RequestBody) -> Self {
            self.body = body;
            self
        }

        pub fn build(self) -> Request {
            let req = self.builder.build();
            req.authentication()
                .set_auth_type(self.authentication.auth_type());
            req.authentication()
                .set_auth_data(self.authentication.auth_data());
            req.body().set_body_type(self.body.body_type());
            req.body().set_body_data(self.body.body_data());
            req
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        RequestAuthentication, RequestAuthenticationBearer, RequestAuthenticationType, RequestBody,
        RequestBodyRaw, RequestBodyRawType, RequestBodyType,
    };

    use super::*;

    #[test]
    pub fn test_valid_builder() {
        let request =
            Request::builder("https://www.example.com/api/users", RequestMethod::Get).build();
        assert_eq!(request.url(), "https://www.example.com/api/users");
        assert_eq!(request.method(), RequestMethod::Get);
        assert_eq!(
            request.authentication().auth_type(),
            RequestAuthenticationType::None
        );
        assert!(request.authentication().auth_data().is_none());
    }

    #[test]
    pub fn test_builder_can_change_authentication() {
        let bearer = RequestAuthenticationBearer::builder().token("1234").build();
        let request = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
            .with_auth(
                RequestAuthentication::builder()
                    .bearer_token(&bearer)
                    .build(),
            )
            .build();
        assert_eq!(request.url(), "https://www.example.com/api/users");
        assert_eq!(request.method(), RequestMethod::Get);
        assert_eq!(
            request.authentication().auth_type(),
            RequestAuthenticationType::BearerToken
        );
        let bearer = request.authentication().bearer_token().unwrap();
        assert_eq!(bearer.token(), "1234");
    }

    #[test]
    pub fn test_builder_can_change_body() {
        let body = RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
            .payload(&glib::Bytes::from(b"hello world"))
            .build();
        let request = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&body).build())
            .build();
        assert_eq!(request.url(), "https://www.example.com/api/users");
        assert_eq!(request.method(), RequestMethod::Get);
        assert_eq!(request.body().body_type(), RequestBodyType::Raw);
        let bearer = request.body().raw().unwrap();
        assert_eq!(bearer.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(bearer.payload().into_data().as_ref(), b"hello world");
    }
}
