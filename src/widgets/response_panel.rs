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
use gtk::gio::{Settings, SettingsBindFlags};
use gtk::prelude::TextViewExt;
use gtk::prelude::*;
use gtk::{glib, WrapMode};

use crate::client::Response;
use glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use glib::object::Cast;
    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{
        subclass::widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
        Box, CompositeTemplate, Label, TemplateChild,
    };
    use gtk::{ScrolledWindow, Spinner, Stack};
    use sourceview5::View;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::ResponsePanel)]
    #[template(resource = "/es/danirod/Cartero/response_panel.ui")]
    pub struct ResponsePanel {
        #[template_child]
        pub response_header_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub response_body: TemplateChild<View>,
        #[template_child]
        pub response_meta: TemplateChild<Box>,
        #[template_child]
        pub status_code: TemplateChild<Label>,
        #[template_child]
        pub duration: TemplateChild<Label>,
        #[template_child]
        pub response_size: TemplateChild<Label>,
        #[template_child]
        pub spinner: TemplateChild<Spinner>,
        #[template_child]
        pub metadata_stack: TemplateChild<Stack>,

        #[property(get = Self::spinning, set = Self::set_spinning)]
        _spinning: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResponsePanel {
        const NAME: &'static str = "CarteroResponsePanel";
        type Type = super::ResponsePanel;
        type ParentType = Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ResponsePanel {}

    impl WidgetImpl for ResponsePanel {}

    impl BoxImpl for ResponsePanel {}

    impl ResponsePanel {
        fn spinning(&self) -> bool {
            self.metadata_stack
                .visible_child()
                .is_some_and(|w| w.is::<Spinner>())
        }

        fn set_spinning(&self, spinning: bool) {
            let widget: &gtk::Widget = if spinning {
                self.spinner.upcast_ref()
            } else {
                self.response_meta.upcast_ref()
            };
            self.metadata_stack.set_visible_child(widget);
        }
    }
}

glib::wrapper! {
    pub struct ResponsePanel(ObjectSubclass<imp::ResponsePanel>)
        @extends gtk::Widget, gtk::Overlay,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ResponsePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponsePanel {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn assign_settings(&self, settings: &Settings) {
        let imp = self.imp();

        let body = imp.response_body.get();
        settings
            .bind("body-wrap", &body, "wrap-mode")
            .flags(SettingsBindFlags::GET)
            .mapping(|variant, _| {
                let enabled = variant.get::<bool>().expect("The variant is not a boolean");
                let mode = match enabled {
                    true => WrapMode::Word,
                    false => WrapMode::None,
                };
                Some(mode.to_value())
            })
            .build();
    }

    pub fn start_request(&self) {
        let imp = self.imp();

        imp.metadata_stack.set_visible_child(&*imp.spinner);
    }

    pub fn assign_from_response(&self, resp: &Response) {
        let imp = self.imp();

        let window = &imp.response_header_window;
        let gtk_box = {
            let gtk_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();

            for (hn, hv) in &resp.headers {
                let row = gtk::Box::default();
                row.set_orientation(gtk::Orientation::Horizontal);
                row.set_spacing(4);
                let name = format!("{}:", hn);
                let name = gtk::Label::builder().label(name).build();
                let value = gtk::Label::builder().label(hv).build();
                row.append(&name);
                row.append(&value);
                gtk_box.append(&row);
            }

            gtk_box
        };
        window.set_child(Some(&gtk_box));

        let status = format!("HTTP {}", resp.status_code);
        imp.status_code.set_text(&status);
        imp.status_code.set_visible(true);

        imp.metadata_stack.set_visible_child(&*imp.response_meta);

        imp.response_body.buffer().set_text(&resp.body_as_str());
    }
}
