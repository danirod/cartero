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

use glib::Object;

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::{subclass::InitializingObject, Properties};
    use gtk::{gio::ListModel, CompositeTemplate, ListBox, TemplateChild};

    use crate::objects::KeyValueItem;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::ResponseHeaders)]
    #[template(resource = "/es/danirod/Cartero/response_headers.ui")]
    pub struct ResponseHeaders {
        #[template_child]
        list_box: TemplateChild<ListBox>,

        #[property(name = "headers", set = Self::set_headers, nullable)]
        headers: RefCell<Option<ListModel>>,
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
    impl ObjectImpl for ResponseHeaders {}

    impl WidgetImpl for ResponseHeaders {}

    impl BinImpl for ResponseHeaders {}

    impl ResponseHeaders {
        fn set_headers(&self, model: Option<ListModel>) {
            match model {
                Some(ref model) => {
                    self.list_box.bind_model(Some(model), |item| {
                        let item = item.downcast_ref::<KeyValueItem>().unwrap();
                        let widget = adw::ActionRow::new();
                        widget.set_title(&item.header_name());
                        widget.set_title_selectable(true);
                        widget.set_subtitle(&item.header_value());
                        widget.set_subtitle_selectable(true);
                        widget.add_css_class("property");
                        widget.upcast::<gtk::Widget>()
                    });
                    self.list_box.set_visible(true);
                }
                None => {
                    self.list_box.unbind_model();
                    self.list_box.set_visible(false);
                }
            }
            *self.headers.borrow_mut() = model;
        }
    }
}

glib::wrapper! {
    pub struct ResponseHeaders(ObjectSubclass<imp::ResponseHeaders>)
        @extends gtk::Widget, adw::Bin;
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
