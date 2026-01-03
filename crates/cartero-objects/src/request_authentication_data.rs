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

use glib::{
    object::{Cast, IsA, ObjectExt},
    subclass::prelude::*,
};
use srtemplate::SrTemplate;

use crate::RequestAuthenticationType;

glib::wrapper! {
    /// The base class to group request authentication payloads.
    ///
    /// This class is abstract and cannot be instantiated. However, subclasses
    /// may provide parameters with information required by some authentication
    /// types in a [RequestAuthentication][super::RequestAuthentication].
    pub struct RequestAuthenticationData(ObjectSubclass<imp::RequestAuthenticationData>);
}

impl RequestAuthenticationData {
    /// A special value that encodes a lack of RequestAuthenticationData.
    ///
    /// You can use this value if you prefer to skip all the generic typing
    /// when using a `Option::None::<RequestAuthenticationData>` in your
    /// constructors, builders and setters.
    pub const NONE: Option<Self> = None::<Self>;
}

mod ffi {
    use srtemplate::SrTemplate;

    use crate::RequestAuthenticationType;

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct Class {
        parent_class: glib::gobject_ffi::GObjectClass,
        pub(super) dup: fn(&super::RequestAuthenticationData) -> super::RequestAuthenticationData,
        pub(super) auth_type: fn(&super::RequestAuthenticationData) -> RequestAuthenticationType,
        pub(super) resolve: fn(
            &super::RequestAuthenticationData,
            &SrTemplate,
        )
            -> Result<super::RequestAuthenticationData, srtemplate::Error>,
        pub(super) rendered_headers: fn(&super::RequestAuthenticationData) -> Vec<(String, String)>,
    }

    unsafe impl glib::subclass::types::ClassStruct for Class {
        type Type = super::imp::RequestAuthenticationData;
    }

    impl std::ops::Deref for Class {
        type Target = glib::gobject_ffi::GObjectClass;

        fn deref(&self) -> &Self::Target {
            &self.parent_class
        }
    }
}

mod imp {
    use std::sync::OnceLock;

    use glib::{subclass::Signal, types::StaticType};
    use srtemplate::SrTemplate;

    use crate::RequestAuthenticationType;

    use super::*;

    #[derive(Default)]
    pub struct RequestAuthenticationData;

    #[glib::object_subclass]
    impl ObjectSubclass for RequestAuthenticationData {
        const NAME: &'static str = "CarteroRequestAuthorizationData";
        const ABSTRACT: bool = true;
        type Type = super::RequestAuthenticationData;
        type Class = super::ffi::Class;

        fn class_init(klass: &mut Self::Class) {
            klass.dup = |obj| obj.imp().dup_default();
            klass.auth_type = |obj| obj.imp().auth_type_default();
            klass.resolve = |obj, tpl| obj.imp().resolve_default(tpl);
            klass.rendered_headers = |obj| obj.imp().rendered_headers_default();
        }
    }

    impl ObjectImpl for RequestAuthenticationData {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("changed")
                    .param_types([String::static_type()])
                    .build()]
            })
        }
    }

    impl RequestAuthenticationData {
        fn dup_default(&self) -> super::RequestAuthenticationData {
            panic!("not implemented");
        }

        fn auth_type_default(&self) -> RequestAuthenticationType {
            panic!("not implemented");
        }

        fn resolve_default(
            &self,
            _: &SrTemplate,
        ) -> Result<super::RequestAuthenticationData, srtemplate::Error> {
            panic!("not implemented");
        }

        fn rendered_headers_default(&self) -> Vec<(String, String)> {
            Vec::new()
        }
    }
}

#[doc(hidden)]
pub trait RequestAuthenticationDataExt: IsA<RequestAuthenticationData> {
    fn dup(&self) -> RequestAuthenticationData {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().dup)(this)
    }

    fn auth_type(&self) -> RequestAuthenticationType {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().auth_type)(this)
    }

    fn resolve(
        &self,
        tpl: &SrTemplate,
    ) -> Result<super::RequestAuthenticationData, srtemplate::Error> {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().resolve)(this, tpl)
    }

    fn rendered_headers(&self) -> Vec<(String, String)> {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().rendered_headers)(this)
    }
}

impl<T: IsA<RequestAuthenticationData>> RequestAuthenticationDataExt for T {}

/// Trait with operations for subclasses of [RequestAuthenticationData].
pub trait RequestAuthenticationDataImpl: ObjectImpl {
    /// Duplicates the given request authentication data.
    fn dup(&self) -> RequestAuthenticationData;

    /// Returns the auth-type associated with this class.
    ///
    /// Returns the specific [RequestAuthenticationType][super::RequestAuthenticationType]
    /// variant where it makes sense to use the current class as an
    /// authentication payload. This is also used during validation when the
    /// auth-type of a [RequestAuthentication][super::RequestAuthentication] changes.
    fn auth_type(&self) -> RequestAuthenticationType;

    fn resolve(
        &self,
        tpl: &SrTemplate,
    ) -> Result<super::RequestAuthenticationData, srtemplate::Error>;

    fn rendered_headers(&self) -> Vec<(String, String)>;
}

#[doc(hidden)]
pub trait RequestAuthenticationDataImplExt: RequestAuthenticationDataImpl {
    fn parent_dup(&self) -> RequestAuthenticationData {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let dup = parent_class.dup;
        dup(unsafe { self.obj().unsafe_cast_ref() })
    }

    fn parent_auth_type(&self) -> RequestAuthenticationType {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let auth_type = parent_class.auth_type;
        auth_type(unsafe { self.obj().unsafe_cast_ref() })
    }

    fn parent_resolve(
        &self,
        tpl: &SrTemplate,
    ) -> Result<super::RequestAuthenticationData, srtemplate::Error> {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let resolve = parent_class.resolve;
        resolve(unsafe { self.obj().unsafe_cast_ref() }, tpl)
    }

    fn parent_rendered_headers(&self) -> Vec<(String, String)> {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let rendered_headers = parent_class.rendered_headers;
        rendered_headers(unsafe { self.obj().unsafe_cast_ref() })
    }
}

impl<T: RequestAuthenticationDataImpl> RequestAuthenticationDataImplExt for T {}

unsafe impl<T: RequestAuthenticationDataImpl> IsSubclassable<T> for RequestAuthenticationData {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class);

        let klass = class.as_mut();
        klass.dup = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestAuthenticationDataImpl::dup(this)
        };
        klass.auth_type = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestAuthenticationDataImpl::auth_type(this)
        };
        klass.resolve = |obj, tpl| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestAuthenticationDataImpl::resolve(this, tpl)
        };
        klass.rendered_headers = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestAuthenticationDataImpl::rendered_headers(this)
        };
    }
}

#[cfg(test)]
mod tests {
    use glib::Object;

    use super::RequestAuthenticationData;

    #[test]
    #[should_panic]
    fn test_data_cannot_be_instanced() {
        Object::new::<RequestAuthenticationData>();
    }
}
