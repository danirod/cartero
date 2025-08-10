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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;

use crate::interop::{LoadResult, SaveResult};

glib::wrapper! {
    pub struct BasePane(ObjectSubclass<imp::BasePane>)
        @extends gtk::Widget, adw::BreakpointBin;
}

mod ffi {
    use super::*;
    use gtk::gio::Action;

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct Class {
        parent_class: adw::ffi::AdwBreakpointBinClass,
        pub(super) lookup_action: fn(&super::BasePane, name: &str) -> Option<Action>,
        // TODO: These should be async, but then I wouldn't know how to type the indirect
        // calls made in the ObjectSubclass impl for imp::BasePane.
        pub(super) load: fn(&super::BasePane) -> LoadResult,
        pub(super) save: fn(&super::BasePane) -> SaveResult,
    }

    unsafe impl glib::subclass::types::ClassStruct for Class {
        type Type = super::imp::BasePane;
    }

    impl std::ops::Deref for Class {
        type Target = adw::ffi::AdwBreakpointBinClass;

        fn deref(&self) -> &Self::Target {
            &self.parent_class
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::BasePane)]
    pub struct BasePane {
        #[property(get, set, nullable)]
        file: RefCell<Option<gio::File>>,
        #[property(get, set)]
        dirty: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BasePane {
        const NAME: &'static str = "CarteroBasePane";
        const ABSTRACT: bool = true;
        type Type = super::BasePane;
        type ParentType = adw::BreakpointBin;
        type Class = super::ffi::Class;

        fn class_init(klass: &mut Self::Class) {
            klass.lookup_action = |obj, name| obj.imp().lookup_action_default(name);
            klass.load = |obj| obj.imp().load_default();
            klass.save = |obj| obj.imp().save_default();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for BasePane {}

    impl WidgetImpl for BasePane {}

    impl BreakpointBinImpl for BasePane {}

    impl BasePane {
        fn lookup_action_default(&self, _name: &str) -> Option<gio::Action> {
            None
        }

        fn load_default(&self) -> LoadResult {
            LoadResult::Anonymous
        }

        fn save_default(&self) -> SaveResult {
            SaveResult::Anonymous
        }
    }
}

#[doc(hidden)]
pub trait BasePaneExt: IsA<BasePane> {
    fn lookup_action(&self, name: &str) -> Option<gio::Action> {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().lookup_action)(this, name)
    }

    fn load(&self) -> LoadResult {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().load)(this)
    }

    fn save(&self) -> SaveResult {
        let this = self.upcast_ref();
        let class = this.class();
        (class.as_ref().save)(this)
    }
}

impl<T: IsA<BasePane>> BasePaneExt for T {}

pub trait BasePaneImpl: BreakpointBinImpl {
    fn lookup_action(&self, _name: &str) -> Option<gio::Action> {
        None
    }

    fn load(&self) -> LoadResult {
        LoadResult::Anonymous
    }

    fn save(&self) -> SaveResult {
        SaveResult::Anonymous
    }
}

#[doc(hidden)]
pub trait BasePaneImplExt: BasePaneImpl {
    fn parent_lookup_action(&self, name: &str) -> Option<gio::Action> {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let lookup_action = parent_class.lookup_action;
        lookup_action(unsafe { self.obj().unsafe_cast_ref() }, name)
    }

    fn parent_load(&self) -> LoadResult {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let parent_load = parent_class.load;
        parent_load(unsafe { self.obj().unsafe_cast_ref() })
    }

    fn parent_save(&self) -> SaveResult {
        let data = Self::type_data();
        let parent_class = unsafe { &*(data.as_ref().parent_class() as *const ffi::Class) };
        let parent_save = parent_class.save;
        parent_save(unsafe { self.obj().unsafe_cast_ref() })
    }
}

impl<T: BasePaneImpl> BasePaneImplExt for T {}

unsafe impl<T: BasePaneImpl> IsSubclassable<T> for BasePane {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class);

        let klass = class.as_mut();
        klass.lookup_action = |obj, name| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            BasePaneImpl::lookup_action(this, name)
        };
        klass.load = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            BasePaneImpl::load(this)
        };
        klass.save = |obj| {
            let this = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            BasePaneImpl::save(this)
        };
    }
}
