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

pub mod ffi {
    use glib::subclass::prelude::*;

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct SerializableInterface {
        parent: glib::gobject_ffi::GTypeInterface,
        pub to_toml: fn(&super::Serializable) -> Result<String, ()>,
        pub from_toml: fn(&super::Serializable, &str) -> Result<(), ()>,
    }

    unsafe impl InterfaceStruct for SerializableInterface {
        type Type = super::iface::Serializable;
    }
}

mod iface {
    use std::sync::OnceLock;

    use glib::prelude::*;
    use glib::subclass::prelude::*;

    use super::ffi::SerializableInterface;

    pub struct Serializable;

    #[glib::object_interface]
    impl ObjectInterface for Serializable {
        const NAME: &'static str = "CarteroSerializable";
        type Interface = SerializableInterface;
        type Prerequisites = (gtk::Widget,);

        fn interface_init(iface: &mut SerializableInterface) {
            iface.from_toml = Serializable::from_toml_default;
            iface.to_toml = Serializable::to_toml_default;
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecBoolean::builder("dirty")
                    .nick("Dirty entity")
                    .blurb("Whether the entity has diverged from the last deserializated state")
                    .default_value(false)
                    .readwrite()
                    .build()]
            })
        }
    }

    impl Serializable {
        fn from_toml_default(_sble: &super::Serializable, _content: &str) -> Result<(), ()> {
            Err(())
        }

        fn to_toml_default(_sble: &super::Serializable) -> Result<String, ()> {
            Err(())
        }
    }
}

use glib::object::IsA;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct Serializable(ObjectInterface<iface::Serializable>) @requires gtk::Widget;
}

#[allow(dead_code)]
pub trait SerializableExt: IsA<gtk::Widget> + IsA<Serializable> {
    fn from_toml(&self, content: &str) -> Result<(), ()> {
        let this = self.upcast_ref::<Serializable>();
        let klass = this.interface::<Serializable>().unwrap();
        (klass.as_ref().from_toml)(this, content)
    }

    fn to_toml(&self) -> Result<String, ()> {
        let this = self.upcast_ref::<Serializable>();
        let klass = this.interface::<Serializable>().unwrap();
        (klass.as_ref().to_toml)(this)
    }

    fn dirty(&self) -> bool {
        self.property("dirty")
    }

    fn set_dirty(&self, dirty: bool) {
        self.set_property("dirty", dirty);
    }
}

impl<T: IsA<gtk::Widget> + IsA<Serializable>> SerializableExt for T {}

pub trait SerializableImpl: ObjectImpl + WidgetImpl {
    fn from_toml(&self, _content: &str) -> Result<(), ()>;
    fn to_toml(&self) -> Result<String, ()>;
}

unsafe impl<T: SerializableImpl> IsImplementable<T> for Serializable {
    fn interface_init(iface: &mut glib::Interface<Self>) {
        let iface = AsMut::as_mut(iface);

        fn from_toml_trampoline<T: ObjectSubclass + SerializableImpl>(
            obj: &super::Serializable,
            content: &str,
        ) -> Result<(), ()> {
            let imp = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            SerializableImpl::from_toml(imp, content)
        }
        iface.from_toml = from_toml_trampoline::<T>;

        fn to_toml_trampoline<T: ObjectSubclass + SerializableImpl>(
            obj: &super::Serializable,
        ) -> Result<String, ()> {
            let imp = unsafe { obj.unsafe_cast_ref::<<T as ObjectSubclass>::Type>().imp() };
            SerializableImpl::to_toml(imp)
        }
        iface.to_toml = to_toml_trampoline::<T>;
    }
}
