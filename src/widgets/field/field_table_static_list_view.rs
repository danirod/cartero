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

    use crate::widgets::field::FieldListBoxStaticRow;

    use super::*;
    use cartero_objects::{Field, FieldTable};
    use glib::{subclass::InitializingObject, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::FieldTableStaticListView)]
    #[template(resource = "/es/danirod/Cartero/field_table_static_list_view.ui")]
    pub struct FieldTableStaticListView {
        #[property(get, set)]
        table: RefCell<FieldTable>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldTableStaticListView {
        const NAME: &'static str = "CarteroFieldTableStaticListView";
        type Type = super::FieldTableStaticListView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FieldTableStaticListView {
        fn constructed(&self) {
            self.parent_constructed();

            self.rebind_table();
            self.obj().connect_table_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.rebind_table();
                }
            ));
        }
    }

    impl WidgetImpl for FieldTableStaticListView {}

    impl BoxImpl for FieldTableStaticListView {}

    #[gtk::template_callbacks]
    impl FieldTableStaticListView {
        fn rebind_table(&self) {
            let table = self.obj().table();
            self.list_box.bind_model(Some(&table), move |obj| {
                let field = obj.downcast_ref::<Field>().unwrap();
                let row = glib::Object::new::<FieldListBoxStaticRow>();
                row.set_field(field);
                let wrapper = gtk::ListBoxRow::builder()
                    .child(&row)
                    .activatable(false)
                    .focusable(false)
                    .selectable(false)
                    .build();
                wrapper.upcast::<gtk::Widget>()
            });
        }
    }
}

glib::wrapper! {
    pub struct FieldTableStaticListView(ObjectSubclass<imp::FieldTableStaticListView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
