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

use glib::Object;

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use cartero_objects::{Field, FieldTable};
    use glib::{subclass::InitializingObject, Properties};
    use gtk::{Box, CompositeTemplate, ListBox, TemplateChild};

    use crate::widgets::field::FieldActionRow;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::ResponseHeaders)]
    #[template(resource = "/es/danirod/Cartero/response_headers.ui")]
    pub struct ResponseHeaders {
        #[template_child]
        list_box: TemplateChild<ListBox>,
        #[template_child]
        placeholder: TemplateChild<Box>,

        #[property(get, set)]
        headers: RefCell<FieldTable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResponseHeaders {
        const NAME: &'static str = "CarteroResponseHeaders";
        type Type = super::ResponseHeaders;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ResponseHeaders {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_placeholders();
            self.rebind_model();

            // Subscribe for updates
            let obj = self.obj();
            obj.connect_headers_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.rebind_model();
                }
            ));
        }
    }

    impl WidgetImpl for ResponseHeaders {}

    impl BinImpl for ResponseHeaders {}

    impl ResponseHeaders {
        fn init_placeholders(&self) {
            self.list_box.set_placeholder(Some(&*self.placeholder));
        }

        fn rebind_model(&self) {
            let headers = self.headers.borrow();
            self.list_box.bind_model(Some(&*headers), |item| {
                let field = item.downcast_ref::<Field>().unwrap();
                let widget = FieldActionRow::default();
                widget.set_field(field);
                widget.upcast::<gtk::Widget>()
            });
        }
    }
}

glib::wrapper! {
    pub struct ResponseHeaders(ObjectSubclass<imp::ResponseHeaders>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ResponseHeaders {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ResponseHeaders {
    fn default() -> Self {
        Object::builder().build()
    }
}
