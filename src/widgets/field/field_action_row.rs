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
use glib::Object;

mod imp {
    use std::cell::RefCell;

    use cartero_objects::Field;
    use glib::{subclass::InitializingObject, Properties};
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::FieldActionRow)]
    #[template(resource = "/es/danirod/Cartero/field_action_row.ui")]
    pub struct FieldActionRow {
        #[property(get, set)]
        field: RefCell<Field>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldActionRow {
        const NAME: &'static str = "CarteroFieldActionRow";
        type Type = super::FieldActionRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FieldActionRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let field = obj.property_expression("field");
            field
                .chain_property::<Field>("key")
                .bind(&*obj, "title", Some(&*obj));
            field
                .chain_property::<Field>("value")
                .bind(&*obj, "subtitle", Some(&*obj));
        }
    }

    impl WidgetImpl for FieldActionRow {}

    impl ListBoxRowImpl for FieldActionRow {}

    impl PreferencesRowImpl for FieldActionRow {}

    impl ActionRowImpl for FieldActionRow {}
}

glib::wrapper! {
    pub struct FieldActionRow(ObjectSubclass<imp::FieldActionRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for FieldActionRow {
    fn default() -> Self {
        Object::new()
    }
}
