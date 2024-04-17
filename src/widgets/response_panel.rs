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
use gtk4::glib;
use gtk4::prelude::TextViewExt;
use gtk4::prelude::*;

use crate::client::Response;
use glib::subclass::types::ObjectSubclassIsExt;

mod imp {
    use glib::subclass::InitializingObject;
    use gtk4::subclass::prelude::*;
    use gtk4::ScrolledWindow;
    use gtk4::{
        subclass::widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
        Box, CompositeTemplate, Label, TemplateChild,
    };
    use sourceview5::View;

    #[derive(CompositeTemplate, Default)]
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

    impl ObjectImpl for ResponsePanel {}

    impl WidgetImpl for ResponsePanel {}

    impl BoxImpl for ResponsePanel {}
}

glib::wrapper! {
    pub struct ResponsePanel(ObjectSubclass<imp::ResponsePanel>)
        @extends gtk4::Widget, gtk4::Overlay,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
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

    pub fn assign_from_response(&self, resp: &Response) {
        let imp = self.imp();

        let window = &imp.response_header_window;
        let gtk_box = {
            let gtk_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .build();

            for (hn, hv) in &resp.headers {
                let row = gtk4::Box::default();
                row.set_orientation(gtk4::Orientation::Horizontal);
                row.set_spacing(4);
                let name = format!("{}:", &*hn);
                let name = gtk4::Label::builder().label(&name).build();
                let value = gtk4::Label::builder().label(&*hv).build();
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

        imp.response_body.buffer().set_text(&resp.body_as_str());
    }
}
