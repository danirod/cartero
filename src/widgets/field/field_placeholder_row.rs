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
    use std::{cell::RefCell, sync::OnceLock};

    use super::*;
    use cartero_objects::{Field, FieldTable};
    use glib::{
        subclass::{InitializingObject, Signal},
        Properties,
    };
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::FieldPlaceholderRow)]
    #[template(resource = "/es/danirod/Cartero/field_placeholder_row.ui")]
    pub struct FieldPlaceholderRow {
        #[template_child]
        key: TemplateChild<gtk::Entry>,
        #[template_child]
        value: TemplateChild<gtk::Entry>,

        #[property(get, set)]
        read_only: RefCell<bool>,
        #[property(get, set)]
        table: RefCell<FieldTable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FieldPlaceholderRow {
        const NAME: &'static str = "CarteroFieldPlaceholderRow";
        type Type = super::FieldPlaceholderRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FieldPlaceholderRow {
        fn constructed(&self) {
            self.parent_constructed();

            let key_delegate = self.key.delegate().unwrap();
            key_delegate.connect_insert_text(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |editable, value, _| {
                    editable.stop_signal_emission_by_name("insert-text");
                    let field = Field::builder().key(value).build();
                    imp.obj().emit_by_name::<()>("insert-field", &[&field]);
                }
            ));

            let value_delegate = self.value.delegate().unwrap();
            value_delegate.connect_insert_text(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |editable, value, _| {
                    editable.stop_signal_emission_by_name("insert-text");
                    let field = Field::builder().value(value).build();
                    imp.obj().emit_by_name::<()>("insert-field", &[&field]);
                }
            ));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("insert-field")
                    .param_types([Field::static_type()])
                    .build()]
            })
        }
    }

    impl WidgetImpl for FieldPlaceholderRow {}

    impl BoxImpl for FieldPlaceholderRow {}
}

glib::wrapper! {
    pub struct FieldPlaceholderRow(ObjectSubclass<imp::FieldPlaceholderRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
