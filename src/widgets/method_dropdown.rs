// Copyright 2024 the Cartero authors
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

use glib::{object::ObjectExt, subclass::types::ObjectSubclassIsExt};

use crate::entities::RequestMethod;

mod imp {
    use std::sync::OnceLock;

    use adw::subclass::prelude::*;
    use glib::{
        object::{Cast, ObjectExt},
        subclass::{InitializingObject, Signal},
    };
    use gtk::{prelude::ListModelExt, CompositeTemplate, StringObject, TemplateChild};

    use crate::entities::RequestMethod;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/method_dropdown.ui")]
    pub struct MethodDropdown {
        #[template_child]
        dropdown: TemplateChild<gtk::DropDown>,

        #[template_child]
        verbs_string_list: TemplateChild<gtk::StringList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MethodDropdown {
        const NAME: &'static str = "CarteroMethodDropdown";
        type Type = super::MethodDropdown;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MethodDropdown {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("changed").build()])
        }
    }

    impl WidgetImpl for MethodDropdown {}

    impl BinImpl for MethodDropdown {}

    #[gtk::template_callbacks]
    impl MethodDropdown {
        #[template_callback]
        fn on_selection_changed(&self) {
            self.obj().emit_by_name::<()>("changed", &[]);
        }

        pub(super) fn request_method(&self) -> RequestMethod {
            let method = self
                .dropdown
                .selected_item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string();
            // Note: we should probably be safe from unwrapping here, since it would
            // be impossible to have a method that is not an acceptable value without
            // completely hacking and wrecking the user interface.
            RequestMethod::try_from(method.as_str()).unwrap()
        }

        pub(super) fn set_request_method(&self, rm: RequestMethod) {
            let verb_to_find = String::from(rm);
            let element_count = self.dropdown.model().unwrap().n_items();
            let target_position = (0..element_count).find(|i| {
                if let Some(verb) = self.verbs_string_list.string(*i) {
                    if verb == verb_to_find {
                        return true;
                    }
                }
                false
            });
            if let Some(pos) = target_position {
                self.dropdown.set_selected(pos);
            }
        }
    }
}

glib::wrapper! {
    pub struct MethodDropdown(ObjectSubclass<imp::MethodDropdown>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable;
}

impl MethodDropdown {
    pub fn connect_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "changed",
            true,
            glib::closure_local!(|ref pane| {
                f(pane);
            }),
        )
    }

    pub fn set_request_method(&self, rm: RequestMethod) {
        self.imp().set_request_method(rm)
    }

    pub fn request_method(&self) -> RequestMethod {
        self.imp().request_method()
    }
}
