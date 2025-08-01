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

    use crate::widgets::field::{FieldListBoxRow, FieldPlaceholderRow};

    use super::*;
    use cartero_objects::{Field, FieldTable};
    use glib::{
        subclass::{InitializingObject, Signal},
        Properties,
    };
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::FieldTableListView)]
    #[template(resource = "/es/danirod/Cartero/field_table_list_view.ui")]
    pub struct FieldTableListView {
        #[property(get, set)]
        table: RefCell<FieldTable>,
        #[property(get, set)]
        show_placeholder: RefCell<bool>,
        #[property(get, set)]
        read_only: RefCell<bool>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        placeholder: TemplateChild<FieldPlaceholderRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldTableListView {
        const NAME: &'static str = "CarteroFieldTableListView";
        type Type = super::FieldTableListView;
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
    impl ObjectImpl for FieldTableListView {
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

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("changed")
                    .param_types([String::static_type()])
                    .build()]
            })
        }
    }

    impl WidgetImpl for FieldTableListView {}

    impl BoxImpl for FieldTableListView {}

    #[gtk::template_callbacks]
    impl FieldTableListView {
        #[template_callback]
        fn on_insert_field(&self, field: &Field) {
            self.obj().table().insert(&field);

            // Get a reference to the last row.
            let last_id = self.obj().table().n_items() - 1;
            if let Some(row) = self.list_box.row_at_index(last_id as i32) {
                let target_prop = if field.key().is_empty() {
                    "value"
                } else {
                    "key"
                };
                let _ = row
                    .child()
                    .unwrap()
                    .activate_action("row.grab-focus", Some(&target_prop.to_variant()));
            }
        }

        fn rebind_table(&self) {
            let table = self.obj().table();
            self.list_box.bind_model(
                Some(&table),
                glib::clone!(
                    #[weak]
                    table,
                    #[upgrade_or_panic]
                    move |obj| {
                        let field = obj.downcast_ref::<Field>().unwrap();
                        let row = glib::Object::new::<FieldListBoxRow>();
                        row.set_field(field);

                        row.connect_closure(
                            "deleted",
                            false,
                            glib::closure_local!(
                                #[strong]
                                table,
                                move |row: FieldListBoxRow| {
                                    let field = row.field();
                                    let maybe_pos = table
                                        .iter::<Field>()
                                        .filter_map(|r| r.ok())
                                        .position(|fd| fd.eq(&field));
                                    if let Some(pos) = maybe_pos {
                                        table.remove(pos as u32);
                                    }
                                }
                            ),
                        );

                        let wrapper = gtk::ListBoxRow::builder()
                            .child(&row)
                            .activatable(false)
                            .focusable(false)
                            .selectable(false)
                            .build();
                        wrapper.upcast::<gtk::Widget>()
                    }
                ),
            );

            table.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak(rename_to = widget)]
                    self,
                    move |_: &FieldTable, param: &str| {
                        widget.obj().emit_by_name::<()>("changed", &[&param]);
                    }
                ),
            );
            // TODO: If there was an old signal, it should be removed.
        }
    }
}

glib::wrapper! {
    pub struct FieldTableListView(ObjectSubclass<imp::FieldTableListView>)
        @extends gtk::Widget, gtk::Box;
}
