// Copyright 2024 the Cartero authors
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

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::entities::RequestPayload;

mod imp {
    use adw::subclass::bin::BinImpl;
    use gtk::subclass::prelude::*;
    use gtk::subclass::widget::WidgetImpl;

    #[derive(Default)]
    pub struct BasePayloadPane;

    #[glib::object_subclass]
    impl ObjectSubclass for BasePayloadPane {
        const NAME: &'static str = "CarteroBasePayloadPane";
        const ABSTRACT: bool = true;

        type Type = super::BasePayloadPane;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for BasePayloadPane {}

    impl WidgetImpl for BasePayloadPane {}

    impl BinImpl for BasePayloadPane {}
}

glib::wrapper! {
    pub struct BasePayloadPane(ObjectSubclass<imp::BasePayloadPane>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable;
}

pub trait BasePayloadPaneExt: IsA<BasePayloadPane> {
    fn payload(&self) -> RequestPayload;

    fn set_payload(&self, payload: &RequestPayload);
}

pub trait BasePayloadPaneImpl: WidgetImpl + ObjectImpl + 'static {}

unsafe impl<T: BasePayloadPaneImpl> IsSubclassable<T> for BasePayloadPane {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
