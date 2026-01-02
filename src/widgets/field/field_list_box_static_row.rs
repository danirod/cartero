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
    use std::cell::{OnceCell, RefCell};

    use super::*;
    use cartero_objects::Field;
    use gettextrs::gettext;
    use glib::{subclass::InitializingObject, BindingGroup, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::FieldListBoxStaticRow)]
    #[template(resource = "/es/danirod/Cartero/field_list_box_static_row.ui")]
    pub struct FieldListBoxStaticRow {
        #[property(get, set)]
        field: RefCell<Field>,
        #[property(get, set)]
        allow_concealing: RefCell<bool>,
        #[property(get, set)]
        concealed: RefCell<bool>,

        #[template_child]
        key: TemplateChild<gtk::Entry>,
        #[template_child]
        value: TemplateChild<gtk::Entry>,
        #[template_child]
        checked: TemplateChild<gtk::CheckButton>,
        #[template_child]
        conceal: TemplateChild<gtk::Button>,

        binding_group: OnceCell<glib::BindingGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldListBoxStaticRow {
        const NAME: &'static str = "CarteroFieldListBoxStaticRow";
        type Type = super::FieldListBoxStaticRow;
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
    impl ObjectImpl for FieldListBoxStaticRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_binding_group();
            self.init_concealing();
        }
    }

    impl WidgetImpl for FieldListBoxStaticRow {}

    impl BoxImpl for FieldListBoxStaticRow {}

    #[gtk::template_callbacks]
    impl FieldListBoxStaticRow {
        fn init_binding_group(&self) {
            let binding_group = BindingGroup::new();
            binding_group
                .bind("key", &*self.key, "text")
                .sync_create()
                .build();
            binding_group
                .bind("value", &*self.value, "text")
                .sync_create()
                .build();
            binding_group
                .bind("active", &*self.checked, "active")
                .sync_create()
                .build();
            binding_group.set_source(Some(&self.obj().field()));
            self.obj().connect_field_notify(glib::clone!(
                #[weak]
                binding_group,
                move |field| {
                    binding_group.set_source(Some(&field.field()));
                }
            ));
            self.binding_group
                .set(binding_group)
                .expect("Couldn't initialise BindingGroup here");
        }

        fn init_concealing(&self) {
            self.obj()
                .bind_property("allow-concealing", &*self.conceal, "sensitive")
                .sync_create()
                .build();
            self.obj()
                .bind_property("allow-concealing", &*self.conceal, "opacity")
                .transform_to(|_, value: &glib::Value| {
                    let allows_conceal = value
                        .get::<bool>()
                        .expect("allow-concealing is of invalid type");
                    if allows_conceal {
                        Some(1.0)
                    } else {
                        Some(0.0)
                    }
                })
                .sync_create()
                .build();

            self.obj()
                .bind_property("concealed", &*self.value, "visibility")
                .sync_create()
                .invert_boolean()
                .build();
            self.obj()
                .bind_property("concealed", &*self.conceal, "icon-name")
                .transform_to(|_, value: &glib::Value| {
                    let concealed = value.get::<bool>().expect("concealed is of invalid type");
                    if concealed {
                        Some("view-reveal")
                    } else {
                        Some("view-conceal")
                    }
                })
                .sync_create()
                .build();
            self.obj()
                .bind_property("concealed", &*self.conceal, "tooltip-text")
                .transform_to(|_, value: &glib::Value| {
                    let concealed = value.get::<bool>().expect("concealed is of invalid type");
                    if concealed {
                        Some(gettext("Show value"))
                    } else {
                        Some(gettext("Hide value"))
                    }
                })
                .sync_create()
                .build();
        }

        #[template_callback]
        fn on_conceal_toggle(&self) {
            let next_conceal = !self.obj().concealed();
            self.obj().set_concealed(next_conceal);
        }
    }
}

glib::wrapper! {
    pub struct FieldListBoxStaticRow(ObjectSubclass<imp::FieldListBoxStaticRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
