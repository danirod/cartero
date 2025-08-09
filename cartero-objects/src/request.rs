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
use srtemplate::SrTemplate;

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

    pub fn template_processor(&self) -> SrTemplate<'static> {
        // Currently only delegates to variables(). In the future may be bound to an environment.
        self.variables().template_processor()
    }
}

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        sync::OnceLock,
    };

    use glib::{subclass::Signal, Properties, SignalGroup};

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
        params_group: OnceCell<SignalGroup>,

        #[property(get, set)]
        headers: RefCell<FieldTable>,
        headers_group: OnceCell<SignalGroup>,

        #[property(get, set)]
        variables: RefCell<FieldTable>,
        variables_group: OnceCell<SignalGroup>,

        #[property(get)]
        authentication: RefCell<RequestAuthentication>,
        authentication_group: OnceCell<SignalGroup>,

        #[property(get)]
        body: RefCell<RequestBody>,
        body_group: OnceCell<SignalGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Request {
        const NAME: &'static str = "CarteroRequest";
        type Type = super::Request;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Request {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_signal_group();

            let obj = self.obj();

            self.obj().connect_url_notify(|request| {
                request.emit_by_name::<()>("changed", &[&"url"]);
            });
            self.obj().connect_method_notify(|request| {
                request.emit_by_name::<()>("changed", &[&"method"]);
            });

            self.params_group
                .get()
                .unwrap()
                .set_target(Some(&obj.params()));
            self.obj().connect_params_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |request| {
                    imp.params_group
                        .get()
                        .unwrap()
                        .set_target(Some(&request.params()));
                    request.emit_by_name::<()>("changed", &[&"params"]);
                }
            ));

            self.headers_group
                .get()
                .unwrap()
                .set_target(Some(&obj.headers()));
            self.obj().connect_headers_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |request| {
                    imp.headers_group
                        .get()
                        .unwrap()
                        .set_target(Some(&request.headers()));
                    request.emit_by_name::<()>("changed", &[&"headers"]);
                }
            ));

            self.variables_group
                .get()
                .unwrap()
                .set_target(Some(&obj.variables()));
            self.obj().connect_variables_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |request| {
                    imp.variables_group
                        .get()
                        .unwrap()
                        .set_target(Some(&request.variables()));
                    request.emit_by_name::<()>("changed", &[&"variables"]);
                }
            ));

            self.authentication_group
                .get()
                .expect("No authentication_group?")
                .set_target(Some(&obj.authentication()));
            obj.connect_authentication_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |request| {
                    imp.authentication_group
                        .get()
                        .unwrap()
                        .set_target(Some(&request.authentication()));
                    request.emit_by_name::<()>("changed", &[&"authentication"]);
                }
            ));

            self.body_group
                .get()
                .expect("No body_group?")
                .set_target(Some(&obj.body()));
            self.obj().connect_body_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |request| {
                    imp.body_group
                        .get()
                        .unwrap()
                        .set_target(Some(&request.body()));
                    request.emit_by_name::<()>("changed", &[&"body"]);
                }
            ));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("changed")
                    .param_types([String::static_type()])
                    .build()]
            })
        }
    }

    impl Request {
        fn init_signal_group(&self) {
            let obj = self.obj();

            let params_group = SignalGroup::new::<FieldTable>();
            params_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &FieldTable, param: &str| {
                        let param = format!("params.{param}");
                        obj.emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            self.params_group.set(params_group).unwrap();

            let headers_group = SignalGroup::new::<FieldTable>();
            headers_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &FieldTable, param: &str| {
                        let param = format!("headers.{param}");
                        obj.emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            self.headers_group.set(headers_group).unwrap();

            let variables_group = SignalGroup::new::<FieldTable>();
            variables_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &FieldTable, param: &str| {
                        let param = format!("variables.{param}");
                        obj.emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            self.variables_group.set(variables_group).unwrap();

            let authentication_group = SignalGroup::new::<RequestAuthentication>();
            authentication_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &RequestAuthentication, param: &str| {
                        let param = format!("authentication.{param}");
                        obj.emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            self.authentication_group.set(authentication_group).unwrap();

            let body_group: SignalGroup = SignalGroup::new::<RequestBody>();
            body_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &RequestBody, param: &str| {
                        let param = format!("body.{param}");
                        obj.emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            self.body_group.set(body_group).unwrap();
        }
    }
}

mod builder {
    use crate::{Field, FieldTable, RequestAuthentication, RequestBody};

    use super::*;
    use glib::object::ObjectBuilder;

    pub struct RequestBuilder {
        builder: ObjectBuilder<'static, Request>,
        authentication: RequestAuthentication,
        body: RequestBody,
        headers: FieldTable,
        params: FieldTable,
        variables: FieldTable,
    }

    impl RequestBuilder {
        pub fn new(url: &str, method: RequestMethod) -> Self {
            let builder = glib::Object::builder()
                .property("url", url)
                .property("method", method);
            let authentication = RequestAuthentication::default();
            let body = RequestBody::default();
            let headers = FieldTable::default();
            let params = FieldTable::default();
            let variables = FieldTable::default();
            Self {
                builder,
                authentication,
                body,
                headers,
                params,
                variables,
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

        pub fn header(self, header: &Field) -> Self {
            self.headers.insert(header);
            self
        }

        pub fn headers(self, headers: &FieldTable) -> Self {
            self.headers.replace(headers);
            self
        }

        pub fn param(self, param: &Field) -> Self {
            self.params.insert(param);
            self
        }

        pub fn params(self, params: &FieldTable) -> Self {
            self.params.replace(params);
            self
        }

        pub fn variable(self, variable: &Field) -> Self {
            self.variables.insert(variable);
            self
        }

        pub fn variables(self, variables: &FieldTable) -> Self {
            self.variables.replace(variables);
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
            req.headers().replace(&self.headers);
            req.params().replace(&self.params);
            req.variables().replace(&self.variables);
            req
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::test::assert_emits_signal, Field, FieldTable, RequestAuthentication,
        RequestAuthenticationBearer, RequestAuthenticationType, RequestBody, RequestBodyRaw,
        RequestBodyRawType, RequestBodyType,
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

        request
            .authentication()
            .set_auth_type(RequestAuthenticationType::BearerToken);
        assert_eq!(
            request.authentication().auth_type(),
            RequestAuthenticationType::BearerToken
        );
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
            .payload("hello world")
            .build();
        let request = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&body).build())
            .build();
        assert_eq!(request.url(), "https://www.example.com/api/users");
        assert_eq!(request.method(), RequestMethod::Get);
        assert_eq!(request.body().body_type(), RequestBodyType::Raw);
        let raw = request.body().raw().unwrap();
        assert_eq!(raw.payload_type(), RequestBodyRawType::OctetStream);
        assert_eq!(raw.payload(), "hello world");
    }

    #[test]
    fn test_builder_without_headers() {
        let body =
            Request::builder("https://www.example.com/api/users", RequestMethod::Get).build();
        assert_eq!(0, body.headers().group_by_key().len());
    }

    #[test]
    pub fn test_builder_can_add_header() {
        let body = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
            .header(
                &(Field::builder()
                    .key("Content-Type")
                    .value("application/xml")
                    .build()),
            )
            .build();
        assert_eq!(1, body.headers().group_by_key().len());
    }

    #[test]
    pub fn test_builder_can_add_header_and_header() {
        let body = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
            .header(
                &(Field::builder()
                    .key("Content-Type")
                    .value("application/xml")
                    .build()),
            )
            .header(
                &(Field::builder()
                    .key("Authorization")
                    .value("Bearer 1234")
                    .build()),
            )
            .build();
        assert_eq!(2, body.headers().group_by_key().len());
    }

    #[test]
    pub fn test_builder_can_add_duplicate_header() {
        let body = Request::builder("https://www.example.com/@profile", RequestMethod::Get)
            .header(
                &(Field::builder()
                    .key("Accept")
                    .value("application/activity+json")
                    .build()),
            )
            .header(
                &(Field::builder()
                    .key("Accept")
                    .value("application/json")
                    .build()),
            )
            .build();

        let headers = body.headers().group_by_key();
        assert_eq!(1, headers.len());

        let accept = headers.get("Accept").unwrap();
        assert_eq!(2, accept.len());
        assert_eq!("application/activity+json", accept[0].value());
        assert_eq!("application/json", accept[1].value());
    }

    #[test]
    pub fn test_builder_can_add_headers() {
        let headers = FieldTable::from_iter(vec![
            Field::builder()
                .key("Accept")
                .value("application/json")
                .build(),
            Field::builder()
                .key("Authorization")
                .value("Bearer 1234")
                .build(),
        ]);
        let body = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
            .headers(&headers)
            .build();
        assert_eq!(2, body.headers().group_by_key().len());
    }

    #[test]
    fn test_template_processor() {
        let request = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
            .variable(
                &Field::builder()
                    .key("API_ROOT")
                    .value("http://localhost:3000")
                    .build(),
            )
            .build();
        let processor = request.template_processor();

        assert!(processor.contains_variable("API_ROOT"));
        assert_eq!(
            processor.render("{{ API_ROOT }}/v1/users").unwrap(),
            "http://localhost:3000/v1/users"
        );
    }

    #[test]
    fn test_template_processor_outlives_the_request() {
        let processor = {
            let request = Request::builder("https://www.example.com/api/users", RequestMethod::Get)
                .variable(
                    &Field::builder()
                        .key("API_ROOT")
                        .value("http://localhost:3000")
                        .build(),
                )
                .build();
            request.template_processor()
        };

        assert!(processor.contains_variable("API_ROOT"));
        assert_eq!(
            processor.render("{{ API_ROOT }}/v1/users").unwrap(),
            "http://localhost:3000/v1/users"
        );
    }

    #[test]
    fn test_emits_signals() {
        let request = Request::builder("", RequestMethod::Get).build();

        assert_emits_signal(&request, "changed", || {
            request.set_url("https://www.example.com")
        });
        assert_emits_signal(&request, "changed", || {
            request.set_method(RequestMethod::Put)
        });

        assert_emits_signal(&request, "changed", || {
            let param = Field::builder().key("a").value("1").build();
            request.params().insert(&param);
        });
        assert_emits_signal(&request, "changed", || {
            request.set_params(FieldTable::default());
        });
        assert_emits_signal(&request, "changed", || {
            let param = Field::builder().key("a").value("1").build();
            request.params().insert(&param);
        });

        assert_emits_signal(&request, "changed", || {
            let header = Field::builder()
                .key("User-Agent")
                .value("Mozilla/5.0")
                .build();
            request.headers().insert(&header);
        });
        assert_emits_signal(&request, "changed", || {
            request.set_headers(FieldTable::default());
        });
        assert_emits_signal(&request, "changed", || {
            let header = Field::builder()
                .key("User-Agent")
                .value("Mozilla/5.0")
                .build();
            request.headers().insert(&header);
        });

        assert_emits_signal(&request, "changed", || {
            let variable = Field::builder()
                .key("API_ROOT")
                .value("http://localhost:3000")
                .build();
            request.variables().insert(&variable);
        });
        assert_emits_signal(&request, "changed", || {
            request.set_variables(FieldTable::default());
        });
        assert_emits_signal(&request, "changed", || {
            let variable = Field::builder()
                .key("API_ROOT")
                .value("http://localhost:3000")
                .build();
            request.variables().insert(&variable);
        });

        assert_emits_signal(&request, "changed", || {
            request
                .authentication()
                .set_auth_type(RequestAuthenticationType::BearerToken);
        });
        assert_emits_signal(&request, "changed", || {
            request
                .authentication()
                .bearer_token()
                .expect("This is not my bearer token!")
                .set_token("12341234");
        });

        assert_emits_signal(&request, "changed", || {
            request.body().set_body_type(RequestBodyType::Multipart);
        });
        assert_emits_signal(&request, "changed", || {
            let field = Field::builder().key("user_id").value("10").build();
            request
                .body()
                .multipart()
                .expect("This is not my multipart!")
                .params()
                .insert(&field);
        });
    }
}
