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
    use std::{cell::RefCell, sync::OnceLock};

    use super::*;
    use cartero_objects::Field;
    use glib::{
        subclass::{InitializingObject, Signal},
        Properties,
    };
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::FieldListBoxStaticRow)]
    #[template(resource = "/es/danirod/Cartero/field_list_box_static_row.ui")]
    pub struct FieldListBoxStaticRow {
        #[property(get, set)]
        field: RefCell<Field>,

        #[template_child]
        key: TemplateChild<gtk::Entry>,
        #[template_child]
        value: TemplateChild<gtk::Entry>,

        binding_group: glib::BindingGroup,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldListBoxStaticRow {
        const NAME: &'static str = "CarteroFieldListBoxStaticRow";
        type Type = super::FieldListBoxStaticRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FieldListBoxStaticRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_binding_group();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("deleted").build()])
        }
    }

    impl WidgetImpl for FieldListBoxStaticRow {}

    impl BoxImpl for FieldListBoxStaticRow {}

    impl FieldListBoxStaticRow {
        fn init_binding_group(&self) {
            self.binding_group
                .bind("key", &*self.key, "text")
                .bidirectional()
                .sync_create()
                .build();
            self.binding_group
                .bind("value", &*self.value, "text")
                .bidirectional()
                .sync_create()
                .build();
            self.binding_group.set_source(Some(&self.obj().field()));
            self.obj().connect_field_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |field| {
                    imp.binding_group.set_source(Some(&field.field()));
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct FieldListBoxStaticRow(ObjectSubclass<imp::FieldListBoxStaticRow>)
        @extends gtk::Widget, gtk::Box;
}
