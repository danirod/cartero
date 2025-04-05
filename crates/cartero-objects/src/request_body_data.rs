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

use glib::{
    object::{Cast, IsA, ObjectExt},
    subclass::prelude::*,
};

use crate::RequestBodyType;

glib::wrapper! {
    pub struct RequestBodyData(ObjectSubclass<imp::RequestBodyData>);
}

impl RequestBodyData {
    pub const NONE: Option<Self> = None::<Self>;
}

mod ffi {
    use crate::RequestBodyType;

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct Class {
        parent_class: glib::gobject_ffi::GObjectClass,
        pub(super) body_type: fn(&super::RequestBodyData) -> RequestBodyType,
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
            klass.body_type = |obj| obj.imp().body_type_default();
        }
    }

    impl ObjectImpl for RequestBodyData {}

    impl RequestBodyData {
        fn body_type_default(&self) -> RequestBodyType {
            panic!("not implemented");
        }
    }
}

pub trait RequestBodyDataExt: IsA<RequestBodyData> {
    fn body_type(&self) -> RequestBodyType {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().body_type)(this)
    }
}

impl<T: IsA<RequestBodyData>> RequestBodyDataExt for T {}

pub trait RequestBodyDataImpl: ObjectImpl {
    fn body_type(&self) -> RequestBodyType;
}

pub trait RequestBodyDataImplExt: RequestBodyDataImpl {
    fn parent_body_type(&self) -> RequestBodyType {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let body_type = parent_class.body_type;
        body_type(unsafe { self.obj().unsafe_cast_ref() })
    }
}

impl<T: RequestBodyDataImpl> RequestBodyDataImplExt for T {}

unsafe impl<T: RequestBodyDataImpl> IsSubclassable<T> for RequestBodyData {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class);

        let klass = class.as_mut();
        klass.body_type = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            RequestBodyDataImpl::body_type(this)
        }
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
