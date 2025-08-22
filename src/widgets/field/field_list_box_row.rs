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
    use gtk::{gio::SimpleAction, CompositeTemplate};

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::FieldListBoxRow)]
    #[template(resource = "/es/danirod/Cartero/field_list_box_row.ui")]
    pub struct FieldListBoxRow {
        #[property(get, set)]
        read_only: RefCell<bool>,

        #[property(get, set)]
        field: RefCell<Field>,

        #[template_child]
        active: TemplateChild<gtk::CheckButton>,
        #[template_child]
        key: TemplateChild<gtk::Entry>,
        #[template_child]
        value: TemplateChild<gtk::Entry>,

        binding_group: glib::BindingGroup,
        action_group: gtk::gio::SimpleActionGroup,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldListBoxRow {
        const NAME: &'static str = "CarteroFieldListBoxRow";
        type Type = super::FieldListBoxRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FieldListBoxRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.init_binding_group();
            self.init_actions();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("deleted").build()])
        }
    }

    impl WidgetImpl for FieldListBoxRow {}

    impl BoxImpl for FieldListBoxRow {}

    impl FieldListBoxRow {
        fn init_binding_group(&self) {
            self.binding_group
                .bind("active", &*self.active, "active")
                .bidirectional()
                .sync_create()
                .build();
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
            self.binding_group
                .bind("masked", &*self.value, "visibility")
                .bidirectional()
                .sync_create()
                .invert_boolean()
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

        fn init_actions(&self) {
            self.obj()
                .insert_action_group("row", Some(&self.action_group));
            let grab_focus = SimpleAction::new("grab-focus", Some(&*String::static_variant_type()));
            grab_focus.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, variant| {
                    if let Some(variant) = variant {
                        let variant = variant.get::<String>().unwrap();
                        match variant.as_ref() {
                            "active" => {
                                imp.active.grab_focus();
                            }
                            "key" => {
                                imp.key.grab_focus_without_selecting();
                                imp.key.set_position(-1);
                            }
                            "value" => {
                                imp.value.grab_focus_without_selecting();
                                imp.value.set_position(-1);
                            }
                            other => {
                                gtk::glib::g_warning!(
                                    "Cartero",
                                    "Cannot focus call row.grab-focus({})",
                                    other
                                );
                            }
                        };
                    }
                }
            ));
            self.action_group.add_action(&grab_focus);

            self.reconfigure_toggle_secret();
            self.obj().connect_field_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.reconfigure_toggle_secret();
                }
            ));
        }

        fn reconfigure_toggle_secret(&self) {
            if self.action_group.has_action("toggle-secret") {
                self.action_group.remove_action("toggle-secret");
            }

            let toggle =
                gtk::gio::PropertyAction::new("toggle-secret", &self.obj().field(), "masked");
            self.action_group.add_action(&toggle);

            let delete = SimpleAction::new("delete", None);
            delete.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.obj().emit_by_name::<()>("deleted", &[]);
                }
            ));
            self.action_group.add_action(&delete);
        }
    }
}

glib::wrapper! {
    pub struct FieldListBoxRow(ObjectSubclass<imp::FieldListBoxRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
