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
use glib::{prelude::*, Object};
use srtemplate::SrTemplate;

use crate::{
    RequestAuthenticationBasic, RequestAuthenticationBearer, RequestAuthenticationData,
    RequestAuthenticationDataExt,
};

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

fn is_appropiate_payload(
    auth_type: RequestAuthenticationType,
    auth_data: Option<&RequestAuthenticationData>,
) -> bool {
    match auth_type {
        RequestAuthenticationType::None | RequestAuthenticationType::Inherit => auth_data.is_none(),
        _ => auth_data.is_some_and(|a| a.auth_type() == auth_type),
    }
}

glib::wrapper! {
    /// A special object to assign authentication information.
    ///
    /// Even though users could manually add an `Authorization` header to their
    /// HTTP requests, this object will make things easier by providing a way
    /// to assign authentication data to an HTTP request.
    ///
    /// For instance, the user interface may display a form to let the user
    /// type the username and the password, and then use this information to
    /// automatically craft the proper HTTP headers before the request is sent.
    ///
    /// ## Auth data and auth type
    ///
    /// There are multiple authentication schemas. In fact, there are
    /// [a lot][mdn-auth]. We aim to support as much as we can, but some of
    /// them are not supported yet.
    ///
    /// Each authentication scheme is linked to a **type**. It is a key of
    /// the [RequestAuthenticationType][super::RequestAuthenticationType] enum.
    /// Use the `auth-type` property of a `RequestAuthentication` to get or
    /// set the current type.
    ///
    /// Some authentication types may require additional inputs, some of them
    /// not. For instance, when the type is set to [`Inherit`][RequestAuthenticationType::Inherit],
    /// no additional inputs is needed, because the semantics are the type
    /// itself. However, some of them do, such as the [`BasicAuth`][RequestAuthenticationType::BasicAuth],
    /// because it requires an username and a password to actually use.
    ///
    /// Therefore, some authentication types are also linked to an
    /// **authentication data**. It is a payload object that has different
    /// extra properties. When a request is being crafted, these properties
    /// are read and used as input to build the extra HTTP headers, parameters
    /// or payload required to fulfill the HTTP request.
    ///
    /// Each payload is a subclass of [RequestAuthenticationData][super::RequestAuthenticationData].
    ///
    /// | Type | Data |
    /// | ---- | ---- |
    /// | [`None`][RequestAuthenticationType::None] | (none) |
    /// | [`Inherit`][RequestAuthenticationType::Inherit] | (none) |
    /// | [`BasicAuth`][RequestAuthenticationType::BasicAuth] | [`RequestAuthenticationBasic`][super::RequestAuthenticationBasic] |
    /// | [`BearerToken`][RequestAuthenticationType::BearerToken] | [`RequestAuthenticationBearer`][super::RequestAuthenticationBearer] |
    ///
    /// Note that `RequestAuthentication` uses the base `RequestAuthenticationData`
    /// object to store the additional payload. You will have to **downcast** to the
    /// proper subtype. There are methods in this class to help with this,
    /// or you can manually use methods such as `.downcast()` or `.and_downcast()`.
    ///
    /// [mdn-auth]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Authentication#authentication_schemes
    ///
    /// ## Parameters
    ///
    /// - `auth-data`: the authentication data object that can be filled with
    ///   information that is specific to the type in use.
    /// - `auth-type`: the authentication type in use.
    ///
    /// ## Creating an Authentication object
    ///
    /// There are two ways.
    ///
    /// - Use the `default()` method to craft a new RequestAuthentication
    ///   object with the authentication type set to
    ///   [NONE][`super::RequestAuthenticationType::None`]. This would
    ///   indicate that there is actually no authorization in use, and it
    ///   would usually be a noop, ignored during an HTTP request and not
    ///   even persisted into a file or exported.
    ///
    /// ```
    /// use cartero_objects::{Request, RequestAuthentication, RequestAuthenticationType};
    ///
    /// let authentication = RequestAuthentication::default();
    /// assert_eq!(RequestAuthenticationType::None, authentication.auth_type());
    /// assert!(authentication.auth_data().is_none());
    /// ```
    ///
    /// - Manually initialise the authentication object with a specific type.
    ///   You can optionally provide also an initial form, but if not given,
    ///   the default will be assumed.
    ///
    /// ```
    /// use cartero_objects::*;
    ///
    /// let authentication = RequestAuthentication::new(RequestAuthenticationType::BasicAuth, RequestAuthenticationData::NONE);
    /// assert_eq!(authentication.auth_type(), RequestAuthenticationType::BasicAuth);
    /// assert!(authentication.auth_data().is_some());
    ///
    /// let token = RequestAuthenticationBearer::new("auth_token");
    /// let authentication = RequestAuthentication::new(RequestAuthenticationType::BearerToken, Some(token));
    /// assert_eq!(authentication.auth_type(), RequestAuthenticationType::BearerToken);
    /// assert!(authentication.auth_data().is_some());
    /// ```
    ///
    /// ## Type checking and restrictions
    ///
    /// As the table in the previous section presents, some authentication types
    /// may require additional payloads. The setters for `RequestAuthentication`
    /// will attempt their best to make both parameters have the same type.
    ///
    /// - If the `auth-type` of a `RequestAuthentication` changes, the `auth-data`
    ///   will be reset to an empty object of the appropiate class for the new
    ///   authentication type. Therefore, the following shall be verified:
    ///
    /// ```
    /// # use cartero_objects::*;
    /// use glib::object::ObjectExt;
    ///
    /// // We start with a request authentication object of type BearerToken.
    /// let token = RequestAuthenticationBearer::new("auth_token");
    /// let authentication = RequestAuthentication::new(RequestAuthenticationType::BearerToken, Some(token));
    ///
    /// // The auth-data is currently a RequestAuthenticationBearer.
    /// assert!(authentication.auth_data().is_some_and(|d| d.is::<RequestAuthenticationBearer>()));
    ///
    /// // However, if we change the authentication type.
    /// authentication.set_auth_type(RequestAuthenticationType::BasicAuth);
    ///
    /// // Then the auth-data is reset to a new object of the appropiate type too.
    /// assert!(authentication.auth_data().is_some_and(|d| d.is::<RequestAuthenticationBasic>()));
    /// ```
    ///
    /// - If the `auth-data` changes to a different object, type checking will
    ///   be applied and if the type of the given `auth-data` is incompatible
    ///   with the current `auth-type`, it will not actually change.
    ///   **I wish the setter would panic instead**, but unfortunately this
    ///   seems to not be supported by gtk-rs. At least the `notify::auth-data`
    ///   signal is not emitted.
    ///
    /// ```
    /// # use cartero_objects::*;
    /// use glib::object::{CastNone, ObjectExt};
    ///
    /// // We start again with a bearer authentication.
    /// let token = RequestAuthenticationBearer::new("auth_token");
    /// let authentication = RequestAuthentication::new(RequestAuthenticationType::BearerToken, Some(token));
    ///
    /// // The auth-data is currently a RequestAuthenticationBearer with the proper value.
    /// assert!(authentication.auth_data().is_some_and(|d| d.is::<RequestAuthenticationBearer>()));
    /// let bearer = authentication.auth_data().and_downcast::<RequestAuthenticationBearer>().unwrap();
    /// assert_eq!("auth_token", bearer.token());
    ///
    /// // You can manually change it to a different object of the same type.
    /// let new_token = RequestAuthenticationBearer::new("next_auth_token");
    /// authentication.set_auth_data(Some(new_token));
    ///
    /// // The auth-data is still valid and changed.
    /// assert!(authentication.auth_data().is_some_and(|d| d.is::<RequestAuthenticationBearer>()));
    /// let bearer = authentication.auth_data().and_downcast::<RequestAuthenticationBearer>().unwrap();
    /// assert_eq!("next_auth_token", bearer.token());
    ///
    /// // However, if we change it to an invalid type.
    /// let basic_auth = RequestAuthenticationBasic::new("root", "toor");
    /// authentication.set_auth_data(Some(basic_auth));
    ///
    /// // Then the auth-data does not change.
    /// assert!(authentication.auth_data().is_some_and(|d| d.is::<RequestAuthenticationBearer>()));
    /// let bearer = authentication.auth_data().and_downcast::<RequestAuthenticationBearer>().unwrap();
    /// assert_eq!("next_auth_token", bearer.token());
    /// ```
    pub struct RequestAuthentication(ObjectSubclass<imp::RequestAuthentication>);
}

impl Default for RequestAuthentication {
    fn default() -> Self {
        Object::new()
    }
}

impl RequestAuthentication {
    /// Creates a new authentication object with the given data.
    ///
    /// The `auth_type` is provided, and optionally an `auth_data` object
    /// can be given if the authentication type supports so, and if there
    /// is initial data to assign. Otherwise, the default data for that
    /// type is used.
    ///
    /// This function does in fact panic if the auth-data has an invalid
    /// type that does not match the given auth-type.
    pub fn new<T>(auth_type: RequestAuthenticationType, auth_data: Option<T>) -> Self
    where
        T: IsA<RequestAuthenticationData>,
    {
        let auth_data: Option<RequestAuthenticationData> = auth_data
            .map(|data| data.upcast())
            .or_else(|| default_authentication_data(auth_type));
        if !is_appropiate_payload(auth_type, auth_data.as_ref()) {
            panic!("auth_data type does not match auth_type");
        }
        Object::builder()
            .property("auth-type", auth_type)
            .property("auth-data", auth_data)
            .build()
    }

    pub fn dup(&self) -> Self {
        Self::new(self.auth_type(), self.auth_data().map(|body| body.dup()))
    }

    pub fn builder() -> builder::RequestAuthenticationBuilder {
        builder::RequestAuthenticationBuilder::default()
    }

    pub fn resolve(&self, tpl: &SrTemplate) -> Result<Self, srtemplate::Error> {
        let auth_type = self.auth_type();
        let auth_data = self.auth_data().map(|data| data.resolve(tpl)).transpose()?;
        Ok(RequestAuthentication::new(auth_type, auth_data))
    }

    /// Returns the basic authentication data, if it's the current type.
    pub fn basic_auth(&self) -> Option<RequestAuthenticationBasic> {
        if self.auth_type() == RequestAuthenticationType::BasicAuth {
            self.auth_data()
                .and_downcast::<RequestAuthenticationBasic>()
        } else {
            None
        }
    }

    /// Returns the bearer token data, if it's the current type.
    pub fn bearer_token(&self) -> Option<RequestAuthenticationBearer> {
        if self.auth_type() == RequestAuthenticationType::BearerToken {
            self.auth_data()
                .and_downcast::<RequestAuthenticationBearer>()
        } else {
            None
        }
    }
}

/// The kind of authentication being used in a [RequestAuthentication] form.
///
/// There are multiple types of authentications, and this enum allows both the
/// user interface and the interoperability libraries to know which one is
/// the one in use for this request.
///
/// Changing the type of a `RequestAuthentication` may change the parameters
/// visible in the screen and may even change the internal state of the
/// authentication object itself in order to accomodate different kind of
/// information.
///
/// Please see the doc for [RequestAuthentication] to know more about the
/// relationship between this enum and the form object.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestAuthenticationType")]
pub enum RequestAuthenticationType {
    /// No authentication in use.
    #[default]
    #[enum_value(name = "NONE", nick = "None")]
    None,

    /// Inherit the authentication from the collection of parent folder.
    #[enum_value(name = "INHERIT", nick = "Inherit")]
    Inherit,

    /// Basic Authentication based on the RFC 7617.
    #[enum_value(name = "BASIC_AUTHENTICATION", nick = "Basic Authentication")]
    BasicAuth,

    /// A special token usually retrieved as a result of other authentication
    /// methods such as OAuth 2.0, based on the RFC 6750.
    #[enum_value(name = "BEARER_TOKEN", nick = "Bearer Token")]
    BearerToken,
}

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        sync::OnceLock,
    };

    use glib::{subclass::Signal, Properties, SignalGroup};

    use crate::RequestAuthenticationData;

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::RequestAuthentication)]
    pub struct RequestAuthentication {
        #[property(get, set = Self::set_auth_type, name = "auth-type", builder(RequestAuthenticationType::None))]
        auth_type: RefCell<RequestAuthenticationType>,

        #[property(get, set = Self::set_auth_data, explicit_notify, name = "auth-data", nullable)]
        auth_data: RefCell<Option<RequestAuthenticationData>>,
        auth_data_group: OnceCell<SignalGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestAuthentication {
        const NAME: &'static str = "CarteroRequestAuthentication";
        type Type = super::RequestAuthentication;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestAuthentication {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_signal_group();

            self.obj().connect_auth_type_notify(|auth| {
                auth.emit_by_name::<()>("changed", &[&"type"]);
            });
            self.obj().connect_auth_data_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |auth| {
                    imp.auth_data_group
                        .get()
                        .unwrap()
                        .set_target(auth.auth_data().as_ref());
                    auth.emit_by_name::<()>("changed", &[&"data"]);
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

    impl RequestAuthentication {
        fn init_signal_group(&self) {
            let obj = self.obj();

            let auth_data_group = SignalGroup::new::<RequestAuthenticationData>();
            auth_data_group.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak]
                    obj,
                    move |_: &RequestAuthenticationData, param: &str| {
                        let param = format!("data.{param}");
                        obj.emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            self.auth_data_group.set(auth_data_group).unwrap();
        }

        // This is the inner setter for the auth-type property. It also changes the auth-data
        // to a new object of the appropiate type. The old contents of the auth-data are erased
        // in the process.
        fn set_auth_type(&self, auth_type: RequestAuthenticationType) {
            let next = default_authentication_data(auth_type);

            let current_type = { self.auth_type.borrow().clone() };
            if current_type != auth_type {
                self.auth_type.replace(auth_type);
                self.obj().set_auth_data(next);
                self.obj().notify_auth_data();
            }
        }

        // This is the inner setter for the auth-data property, which also verifies that the
        // type of the given data is acceptable for the current auth-type the object is set to.
        fn set_auth_data(&self, auth_data: Option<RequestAuthenticationData>) {
            let current_type = self.obj().auth_type();
            if is_appropiate_payload(current_type, auth_data.as_ref()) {
                self.auth_data.replace(auth_data);
                self.obj().notify_auth_data();
            } else {
                #[cfg(not(test))]
                glib::g_critical!("Cartero", "set_auth_data() was called with a RequestAuthenticationData of invalid RequestAuthenticationType for this RequestAuthentication object");
            }
        }
    }
}

mod builder {
    use super::*;
    use glib::object::ObjectBuilder;

    pub struct RequestAuthenticationBuilder {
        builder: ObjectBuilder<'static, RequestAuthentication>,
    }

    impl Default for RequestAuthenticationBuilder {
        fn default() -> Self {
            let builder = Object::builder();
            Self { builder }
        }
    }

    impl RequestAuthenticationBuilder {
        pub fn build(self) -> RequestAuthentication {
            self.builder.build()
        }

        pub fn none(mut self) -> Self {
            self.builder = self
                .builder
                .property("auth-type", RequestAuthenticationType::None);
            self
        }

        pub fn inherit(mut self) -> Self {
            self.builder = self
                .builder
                .property("auth-type", RequestAuthenticationType::Inherit);
            self
        }

        pub fn basic_auth(mut self, basic_auth: &RequestAuthenticationBasic) -> Self {
            self.builder = self
                .builder
                .property("auth-type", RequestAuthenticationType::BasicAuth)
                .property("auth-data", Some(basic_auth));
            self
        }

        pub fn bearer_token(mut self, bearer: &RequestAuthenticationBearer) -> Self {
            self.builder = self
                .builder
                .property("auth-type", RequestAuthenticationType::BearerToken)
                .property("auth-data", Some(bearer));
            self
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
    pub fn builder_default() {
        let auth = RequestAuthentication::builder().build();
        assert_eq!(auth.auth_type(), RequestAuthenticationType::None);
        assert!(auth.auth_data().is_none());
    }

    #[test]
    pub fn builder_inherit() {
        let auth = RequestAuthentication::builder().inherit().build();
        assert_eq!(auth.auth_type(), RequestAuthenticationType::Inherit);
        assert!(auth.auth_data().is_none());
    }

    #[test]
    pub fn builder_basic_auth() {
        let basic = RequestAuthenticationBasic::builder()
            .username("user")
            .password("pass")
            .build();
        let auth = RequestAuthentication::builder().basic_auth(&basic).build();
        assert_eq!(auth.auth_type(), RequestAuthenticationType::BasicAuth);
        let basic = auth.basic_auth().unwrap();
        assert_eq!(basic.username(), "user");
        assert_eq!(basic.password(), "pass");
    }

    #[test]
    pub fn builder_bearer() {
        let bearer = RequestAuthenticationBearer::builder()
            .token("aabbccdd")
            .build();
        let auth = RequestAuthentication::builder()
            .bearer_token(&bearer)
            .build();
        assert_eq!(auth.auth_type(), RequestAuthenticationType::BearerToken);
        let bearer = auth.bearer_token().unwrap();
        assert_eq!(bearer.token(), "aabbccdd");
    }

    #[test]
    pub fn new_default() {
        let authentication = RequestAuthentication::default();
        assert_eq!(RequestAuthenticationType::None, authentication.auth_type());
        assert!(authentication.auth_data().is_none());
    }

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

        {
            let auth_data = authentication.auth_data().unwrap();
            assert_eq!(RequestAuthenticationType::BasicAuth, auth_data.auth_type());
            let basic_auth_data = auth_data.downcast::<RequestAuthenticationBasic>().unwrap();
            assert_eq!(basic_auth_data.username(), "admin");
            assert_eq!(basic_auth_data.password(), "1234");
        }

        authentication.set_auth_type(RequestAuthenticationType::BasicAuth);

        {
            let auth_data = authentication.auth_data().unwrap();
            assert_eq!(RequestAuthenticationType::BasicAuth, auth_data.auth_type());
            let basic_auth_data = auth_data.downcast::<RequestAuthenticationBasic>().unwrap();
            assert_eq!(basic_auth_data.username(), "admin");
            assert_eq!(basic_auth_data.password(), "1234");
        }
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

    #[test]
    pub fn test_emits_signal_on_change() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::None,
            RequestAuthenticationData::NONE,
        );
        assert_emits_signal(&auth, "changed", || {
            auth.set_auth_type(RequestAuthenticationType::Inherit);
        });
        assert_eq!(auth.auth_type(), RequestAuthenticationType::Inherit);
        assert_emits_signal(&auth, "changed", || {
            auth.set_auth_type(RequestAuthenticationType::BearerToken);
        });
        assert_eq!(auth.auth_type(), RequestAuthenticationType::BearerToken);
    }

    #[test]
    pub fn test_emits_signal_on_basic_change() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BasicAuth,
            RequestAuthenticationData::NONE,
        );
        assert_emits_signal(&auth, "changed", || {
            auth.basic_auth().unwrap().set_username("foo");
        });
        assert_emits_signal(&auth, "changed", || {
            auth.basic_auth().unwrap().set_password("bar");
        });
    }

    #[test]
    pub fn test_emits_signal_on_bearer_change() {
        let auth = RequestAuthentication::new(
            RequestAuthenticationType::BearerToken,
            RequestAuthenticationData::NONE,
        );
        assert_emits_signal(&auth, "changed", || {
            auth.bearer_token().unwrap().set_token("12341234");
        });
    }

    #[test]
    fn test_resolve_when_auth_is_none() {
        let auth = RequestAuthentication::builder().none().build();
        let tpl = SrTemplate::default();
        let resolved_auth = auth.resolve(&tpl).expect("Invalid resolve?");
        assert_eq!(resolved_auth.auth_type(), RequestAuthenticationType::None);
        assert!(resolved_auth.auth_data().is_none());
    }

    #[test]
    fn test_resolve_when_auth_is_basic() {
        let auth = RequestAuthenticationBasic::builder()
            .username("{{USER}}")
            .password("{{PASS}}")
            .build();
        let auth = RequestAuthentication::builder().basic_auth(&auth).build();
        let tpl = SrTemplate::default();
        tpl.add_variable("USER", "admin");
        tpl.add_variable("PASS", "1234");
        let resolved_auth = auth.resolve(&tpl).expect("Invalid resolve?");
        assert_eq!(
            resolved_auth.auth_type(),
            RequestAuthenticationType::BasicAuth
        );
        let resolved_basic = resolved_auth.basic_auth().expect("No basic auth?");
        assert_eq!(resolved_basic.username(), "admin");
        assert_eq!(resolved_basic.password(), "1234");
    }

    #[test]
    fn test_resolve_when_auth_is_bearer() {
        let auth = RequestAuthenticationBearer::builder()
            .token("{{TOKEN}}")
            .build();
        let auth = RequestAuthentication::builder().bearer_token(&auth).build();
        let tpl = SrTemplate::default();
        tpl.add_variable("TOKEN", "12341234");
        let resolved_auth = auth.resolve(&tpl).expect("Invalid resolve?");
        assert_eq!(
            resolved_auth.auth_type(),
            RequestAuthenticationType::BearerToken
        );
        let resolved_bearer = resolved_auth.bearer_token().expect("No bearer token?");
        assert_eq!(resolved_bearer.token(), "12341234");
    }
}
