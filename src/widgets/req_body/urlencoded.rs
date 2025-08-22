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

mod imp {
    use std::cell::RefCell;

    use super::*;
    use cartero_objects::FieldTable;
    use glib::{subclass::InitializingObject, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Urlencoded)]
    #[template(resource = "/es/danirod/Cartero/urlencoded_body_pane.ui")]
    pub struct Urlencoded {
        #[property(get, set)]
        table: RefCell<FieldTable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Urlencoded {
        const NAME: &'static str = "CarteroUrlencodedBodyPane";
        type Type = super::Urlencoded;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Urlencoded {}

    impl WidgetImpl for Urlencoded {}

    impl BinImpl for Urlencoded {}
}

glib::wrapper! {
    pub struct Urlencoded(ObjectSubclass<imp::Urlencoded>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
