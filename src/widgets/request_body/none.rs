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

use super::BasePayloadPane;

mod imp {
    use std::cell::RefCell;

    use adw::subclass::bin::BinImpl;
    use glib::Properties;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::{
        entities::KeyValueTable,
        widgets::{BasePayloadPane, BasePayloadPaneImpl},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::NonePayloadPane)]
    pub struct NonePayloadPane {
        #[property(get = Self::payload, set = Self::set_payload, nullable, type = Option<glib::Bytes>)]
        _payload: RefCell<Option<glib::Bytes>>,

        #[property(get = Self::headers, type = KeyValueTable)]
        _headers: RefCell<KeyValueTable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NonePayloadPane {
        const NAME: &'static str = "CarteroNonePayloadPane";
        type Type = super::NonePayloadPane;
        type ParentType = BasePayloadPane;
    }

    #[glib::derived_properties]
    impl ObjectImpl for NonePayloadPane {}

    impl WidgetImpl for NonePayloadPane {}

    impl BinImpl for NonePayloadPane {}

    impl BasePayloadPaneImpl for NonePayloadPane {}

    impl NonePayloadPane {
        fn headers(&self) -> KeyValueTable {
            KeyValueTable::default()
        }

        fn payload(&self) -> Option<glib::Bytes> {
            None
        }

        fn set_payload(&self, _: Option<&glib::Bytes>) {}
    }
}

glib::wrapper! {
    pub struct NonePayloadPane(ObjectSubclass<imp::NonePayloadPane>)
        @extends gtk::Widget, adw::Bin, BasePayloadPane,
        @implements gtk::Accessible;
}
