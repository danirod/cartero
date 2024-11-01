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

use crate::entities::RequestExportType;

mod imp {
    use adw::subclass::bin::BinImpl;
    use gtk::subclass::prelude::*;
    use gtk::subclass::widget::WidgetImpl;

    #[derive(Default)]
    pub struct BaseExportPane;

    #[glib::object_subclass]
    impl ObjectSubclass for BaseExportPane {
        const NAME: &'static str = "CarteroBaseExportPane";
        const ABSTRACT: bool = true;

        type Type = super::BaseExportPane;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for BaseExportPane {}

    impl WidgetImpl for BaseExportPane {}

    impl BinImpl for BaseExportPane {}
}

glib::wrapper! {
    pub struct BaseExportPane(ObjectSubclass<imp::BaseExportPane>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable;
}

pub trait BaseExportPaneExt: IsA<BaseExportPane> {
    fn request_export_type(&self) -> RequestExportType;

    fn set_request_export_type(&self, req_export_type: &RequestExportType);
}

pub trait BaseExportPaneImpl: WidgetImpl + ObjectImpl + 'static {}

unsafe impl<T: BaseExportPaneImpl> IsSubclassable<T> for BaseExportPane {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
