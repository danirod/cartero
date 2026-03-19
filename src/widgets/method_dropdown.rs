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

use cartero_objects::RequestMethod;

mod imp {
    use super::*;
    use adw::EnumListItem;
    use glib::{Properties, subclass::InitializingObject};
    use gtk::{ClosureExpression, CompositeTemplate};
    use std::cell::RefCell;

    #[derive(Default, CompositeTemplate, Properties)]
    #[template(resource = "/es/danirod/Cartero/method_dropdown.ui")]
    #[properties(wrapper_type = super::MethodDropdown)]
    pub struct MethodDropdown {
        #[template_child]
        dropdown: TemplateChild<gtk::DropDown>,

        #[template_child]
        model: TemplateChild<adw::EnumListModel>,

        #[property(get, set, builder(RequestMethod::default()))]
        request_method: RefCell<RequestMethod>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MethodDropdown {
        const NAME: &'static str = "CarteroMethodDropdown";
        type Type = super::MethodDropdown;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            RequestMethod::static_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MethodDropdown {
        fn constructed(&self) {
            self.parent_constructed();

            let expr = ClosureExpression::with_callback(gtk::Expression::NONE, |args| {
                let repr = args[0].get::<EnumListItem>().unwrap();
                repr.name()
            });
            self.dropdown.set_expression(Some(&expr));

            let obj = self.obj();
            obj.bind_property("request-method", &*self.dropdown, "selected")
                .transform_from(|_, value: u32| RequestMethod::try_from(value).ok())
                .transform_to(|_, value: RequestMethod| Some((value as i32) as u32))
                .bidirectional()
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for MethodDropdown {}

    impl BinImpl for MethodDropdown {}

    #[gtk::template_callbacks]
    impl MethodDropdown {}
}

glib::wrapper! {
    pub struct MethodDropdown(ObjectSubclass<imp::MethodDropdown>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MethodDropdown {}
