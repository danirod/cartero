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

mod imp {
    use std::cell::RefCell;

    use crate::widgets::field::FieldListBoxStaticRow;

    use super::*;

    use cartero_objects::{Field, FieldTable};
    use glib::{Properties, subclass::InitializingObject};
    use gtk::CompositeTemplate;

    #[derive(Default, Properties, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/collapsed_field_table.ui")]
    #[properties(wrapper_type = super::CollapsedFieldTable)]
    pub struct CollapsedFieldTable {
        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        subtitle: RefCell<String>,
        #[property(get, set)]
        expanded: RefCell<bool>,
        #[property(get, set)]
        masked: RefCell<bool>,
        #[property(get, set)]
        field_table: RefCell<FieldTable>,

        #[template_child]
        expander: TemplateChild<adw::ExpanderRow>,

        nodes: RefCell<Vec<gtk::Widget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollapsedFieldTable {
        const NAME: &'static str = "CarteroCollapsedFieldTable";
        type Type = super::CollapsedFieldTable;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            FieldListBoxStaticRow::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CollapsedFieldTable {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_table();
            self.obj().connect_field_table_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_table();
                }
            ));
            self.obj().field_table().connect_closure(
                "items-changed",
                false,
                glib::closure_local!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_: glib::Object, _: u32, _: u32, _: u32| {
                        imp.update_table();
                    }
                ),
            );
        }
    }

    impl WidgetImpl for CollapsedFieldTable {}

    impl BinImpl for CollapsedFieldTable {}

    impl CollapsedFieldTable {
        fn update_table(&self) {
            let mut nodes = self.nodes.borrow_mut();

            // Remove existing nodes.
            for node in &*nodes {
                let parent = node.parent();
                let parent = parent.as_ref().unwrap_or(node);
                self.expander.remove(parent);
            }
            nodes.clear();

            // Reset the list.
            for field in self.obj().field_table().iter::<Field>() {
                if let Ok(field) = field {
                    let row: FieldListBoxStaticRow =
                        glib::Object::builder().property("field", field).build();
                    self.obj()
                        .bind_property("masked", &row, "allow-concealing")
                        .sync_create()
                        .build();
                    self.obj()
                        .bind_property("masked", &row, "concealed")
                        .sync_create()
                        .build();
                    let widget = row.upcast();
                    self.expander.add_row(&widget);
                    nodes.push(widget);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct CollapsedFieldTable(ObjectSubclass<imp::CollapsedFieldTable>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
