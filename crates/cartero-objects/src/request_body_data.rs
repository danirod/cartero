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

use crate::RequestBodyType;

glib::wrapper! {
    /// The base class to group request body payloads.
    ///
    /// This class is abstract and cannot be instantiated. However, subclasses
    /// may provide parameters and custom semantics on how to encode and
    /// decode the payload when issuing an HTTP request. See the
    /// [RequestBody][super::RequestBody] for more information.
    pub struct RequestBodyData(ObjectSubclass<imp::RequestBodyData>);
}

impl RequestBodyData {
    /// A special value that encodes a lack of RequestBodyData.
    ///
    /// You can use this value if you prefer to skip all the generic typing
    /// when using a `Option::None::<RequestBodyData>` in your constructors,
    /// builders and setters.
    pub const NONE: Option<Self> = None::<Self>;
}

mod ffi {
    use crate::RequestBodyType;

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct Class {
        parent_class: glib::gobject_ffi::GObjectClass,
        pub(super) dup: fn(&super::RequestBodyData) -> super::RequestBodyData,
        pub(super) body_type: fn(&super::RequestBodyData) -> RequestBodyType,
        pub(super) resolve: fn(
            &super::RequestBodyData,
            &srtemplate::SrTemplate,
        ) -> Result<super::RequestBodyData, srtemplate::Error>,
        pub(super) rendered_headers: fn(&super::RequestBodyData) -> Vec<(String, String)>,
    }

    unsafe impl glib::subclass::types::ClassStruct for Class {
        type Type = super::imp::RequestBodyData;
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

    use crate::RequestBodyType;

    use super::*;

    #[derive(Default)]
    pub struct RequestBodyData;

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyData {
        const NAME: &'static str = "CarteroRequestBodyData";
        const ABSTRACT: bool = true;
        type Type = super::RequestBodyData;
        type Class = super::ffi::Class;

        fn class_init(klass: &mut Self::Class) {
            klass.dup = |obj| obj.imp().dup_default();
            klass.body_type = |obj| obj.imp().body_type_default();
            klass.resolve = |obj, tpl| obj.imp().resolve_default(tpl);
            klass.rendered_headers = |obj| obj.imp().rendered_headers_default();
        }
    }

    impl ObjectImpl for RequestBodyData {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("changed")
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl RequestBodyData {
        fn dup_default(&self) -> super::RequestBodyData {
            panic!("not implemented");
        }

        fn body_type_default(&self) -> RequestBodyType {
            panic!("not implemented");
        }

        fn resolve_default(
            &self,
            _: &SrTemplate,
        ) -> Result<super::RequestBodyData, srtemplate::Error> {
            panic!("not implemented");
        }

        fn rendered_headers_default(&self) -> Vec<(String, String)> {
            Vec::new()
        }
    }
}

#[doc(hidden)]
pub trait RequestBodyDataExt: IsA<RequestBodyData> {
    fn dup(&self) -> RequestBodyData {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().dup)(this)
    }

    fn body_type(&self) -> RequestBodyType {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().body_type)(this)
    }

    fn resolve(&self, tpl: &SrTemplate) -> Result<RequestBodyData, srtemplate::Error> {
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

impl<T: IsA<RequestBodyData>> RequestBodyDataExt for T {}

/// Trait with operations for subclasses of [RequestBodyData].
pub trait RequestBodyDataImpl: ObjectImpl {
    fn dup(&self) -> RequestBodyData;

    /// Returns the body-type associated with this class.
    ///
    /// Returns the specific [RequestBodyType][super::RequestBodyType] variant
    /// where it makes sense to use the current class as a body payload. This
    /// is also used during validation when the body-type of a
    /// [RequestBody][super::RequestBody] changes.
    fn body_type(&self) -> RequestBodyType;

    fn resolve(&self, tpl: &SrTemplate) -> Result<RequestBodyData, srtemplate::Error>;

    fn rendered_headers(&self) -> Vec<(String, String)>;
}

#[doc(hidden)]
pub trait RequestBodyDataImplExt: RequestBodyDataImpl {
    fn parent_dup(&self) -> RequestBodyData {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let dup = parent_class.dup;
        unsafe { dup(self.obj().unsafe_cast_ref()) }
    }

    fn parent_body_type(&self) -> RequestBodyType {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let body_type = parent_class.body_type;
        unsafe { body_type(self.obj().unsafe_cast_ref()) }
    }

    fn resolve(&self, tpl: &SrTemplate) -> Result<RequestBodyData, srtemplate::Error> {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let resolve = parent_class.resolve;
        unsafe { resolve(self.obj().unsafe_cast_ref(), tpl) }
    }

    fn parent_rendered_headers(&self) -> Vec<(String, String)> {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let rendered_headers = parent_class.rendered_headers;
        unsafe { rendered_headers(self.obj().unsafe_cast_ref()) }
    }
}

#[doc(hidden)]
impl<T: RequestBodyDataImpl> RequestBodyDataImplExt for T {}

unsafe impl<T: RequestBodyDataImpl> IsSubclassable<T> for RequestBodyData {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class);

        let klass = class.as_mut();
        klass.dup = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestBodyDataImpl::dup(this)
        };
        klass.body_type = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestBodyDataImpl::body_type(this)
        };
        klass.resolve = |obj, tpl| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestBodyDataImpl::resolve(this, tpl)
        };
        klass.rendered_headers = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestBodyDataImpl::rendered_headers(this)
        };
    }
}

#[cfg(test)]
mod tests {
    use glib::Object;

    use super::RequestBodyData;

    #[test]
    #[should_panic]
    fn test_data_cannot_be_instanced() {
        Object::new::<RequestBodyData>();
    }
}
