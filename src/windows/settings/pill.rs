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

use adw::prelude::*;
use adw::subclass::prelude::*;

glib::wrapper! {
    pub struct Pill(ObjectSubclass<imp::Pill>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

mod imp {
    use std::cell::RefCell;

    use super::*;

    use glib::{subclass::InitializingObject, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate, Properties)]
    #[template(resource = "/es/danirod/Cartero/settings/pill.ui")]
    #[properties(wrapper_type = super::Pill)]
    pub struct Pill {
        #[property(get, set)]
        icon_name: RefCell<String>,
        #[property(get, set)]
        label: RefCell<String>,
        #[property(get, set)]
        name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Pill {
        const NAME: &'static str = "CarteroSettingsPill";
        type Type = super::Pill;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Pill {}

    impl WidgetImpl for Pill {}

    impl BoxImpl for Pill {}
}
