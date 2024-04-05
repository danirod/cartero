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

use crate::app::CarteroApplication;
use glib::{GString, Object};
use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::{gio, glib, StringObject};

mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io::Read;

    use gtk4::gio::{ListModel, ListStore};
    use gtk4::subclass::prelude::*;
    use gtk4::{prelude::*, NoSelection};

    use crate::client::build_request;
    use crate::client::{Request, RequestMethod};
    use crate::components::response_panel::ResponsePanel;
    use crate::components::row_header::RowHeader;
    use crate::config::VERSION;
    use crate::objects::Header;
    use glib::subclass::InitializingObject;
    use gtk4::{
        subclass::{
            application_window::ApplicationWindowImpl, widget::WidgetImpl, window::WindowImpl,
        },
        CompositeTemplate, TemplateChild,
    };
    use isahc::RequestExt;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/main_window.ui")]
    pub struct CarteroWindow {
        #[template_child(id = "send")]
        pub send_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub request_headers: TemplateChild<gtk4::ListView>,

        #[template_child(id = "method")]
        pub request_method: TemplateChild<gtk4::DropDown>,

        #[template_child(id = "url")]
        pub request_url: TemplateChild<gtk4::Entry>,

        #[template_child]
        pub request_body: TemplateChild<sourceview5::View>,

        #[template_child]
        pub response: TemplateChild<ResponsePanel>,
    }

    #[gtk4::template_callbacks]
    impl CarteroWindow {
        #[template_callback]
        fn on_send_request(&self, _: &gtk4::Button) {
            let headers = self.get_headers();
            for h in headers {
                println!("{:?}", h);
            }
        }

        fn get_headers(&self) -> Vec<(String, String)> {
            let mut headers = Vec::new();
            if let Some(model) = self.request_headers.model() {
                let no_selection = model.downcast::<NoSelection>().unwrap();
                let list_model = no_selection.model().unwrap();
                for item in &list_model {
                    if let Ok(thing) = item {
                        let header = thing.downcast::<Header>().unwrap();
                        let value = (header.header_name(), header.header_value());
                        headers.push(value);
                    }
                }
            }
            headers
        }

        fn add_header(&self, h: &Header) {
            if let Some(model) = self.request_headers.model() {
                let no_selection = model.downcast::<NoSelection>().unwrap();
                let list_model = no_selection
                    .model()
                    .unwrap()
                    .downcast::<ListStore>()
                    .unwrap();
                list_model.append(h);
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroWindow {
        const NAME: &'static str = "CarteroWindow";
        type Type = super::CarteroWindow;
        type ParentType = gtk4::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            RowHeader::static_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CarteroWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let model = ListStore::new::<Header>();
            let selection_model = gtk4::NoSelection::new(Some(model.upcast::<ListModel>()));
            self.request_headers.set_model(Some(&selection_model));

            self.add_header(&Header::new("Accept", "text/html"));
            let h = Header::new("Content-Type", "text/html");
            h.set_active(false);
            self.add_header(&h);
            self.add_header(&Header::new("Authorization", "Bearer roar"));
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {}

    impl ApplicationWindowImpl for CarteroWindow {}
}

glib::wrapper! {
    pub struct CarteroWindow(ObjectSubclass<imp::CarteroWindow>)
        @extends gtk4::Widget, gtk4::Window, gtk4::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CarteroWindow {
    pub fn new(app: &CarteroApplication) -> Self {
        Object::builder().property("application", Some(app)).build()
    }

    pub fn request_url(&self) -> GString {
        self.imp().request_url.text()
    }

    pub fn request_method(&self) -> GString {
        let method = &self.imp().request_method;
        method
            .selected_item()
            .unwrap()
            .downcast::<StringObject>()
            .unwrap()
            .string()
    }

    pub fn request_body(&self) -> GString {
        let body = &self.imp().request_body;
        let (start, end) = body.buffer().bounds();
        body.buffer().text(&start, &end, true)
    }
}
