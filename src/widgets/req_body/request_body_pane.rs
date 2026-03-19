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
    use std::cell::RefCell;

    use crate::widgets::{
        req_body::{File, Multipart, Raw, Urlencoded},
        shell::BasePane,
    };

    use super::*;
    use cartero_objects::{
        FieldTable, RequestBody, RequestBodyFile, RequestBodyRawType, RequestBodyType,
    };
    use glib::{Properties, subclass::InitializingObject};
    use gtk::CompositeTemplate;

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::RequestBodyPane)]
    #[template(resource = "/es/danirod/Cartero/request_body_pane.ui")]
    pub struct RequestBodyPane {
        #[property(get, set)]
        body: RefCell<RequestBody>,
        #[property(get, set)]
        read_only: RefCell<bool>,

        last_parameters_table: RefCell<FieldTable>,
        last_raw_payload: RefCell<String>,
        last_file: RefCell<RequestBodyFile>,

        #[template_child]
        container: TemplateChild<adw::Bin>,
        #[template_child]
        combo: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestBodyPane {
        const NAME: &'static str = "CarteroRequestBodyPane";
        type Type = super::RequestBodyPane;
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
    impl ObjectImpl for RequestBodyPane {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_body_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |pane: &super::RequestBodyPane| {
                    // This will eventually call the notify::selected signal of the combo.
                    let body_type = pane.body().body_type();
                    let raw_body_type = pane.body().raw().map(|raw| raw.payload_type());
                    imp.combo
                        .set_selected(cast_entry_to_select(body_type, raw_body_type));
                }
            ));
        }
    }

    impl WidgetImpl for RequestBodyPane {}

    impl BoxImpl for RequestBodyPane {}

    #[gtk::template_callbacks]
    impl RequestBodyPane {
        fn sync_container(&self) {
            let body = self.obj().body();
            let container_child =
                match body.body_type() {
                    RequestBodyType::UrlEncoded => {
                        let urlencoded = body.urlencoded().unwrap();
                        let widget = glib::Object::builder::<Urlencoded>()
                            .property("table", urlencoded.params())
                            .build();
                        Some(widget.upcast::<gtk::Widget>())
                    }
                    RequestBodyType::Multipart => {
                        let urlencoded = body.multipart().unwrap();
                        let widget = glib::Object::builder::<Multipart>()
                            .property("table", urlencoded.params())
                            .build();
                        Some(widget.upcast::<gtk::Widget>())
                    }
                    RequestBodyType::Raw => {
                        let raw = body.raw().unwrap();
                        let widget = glib::Object::builder::<Raw>()
                            .property("payload", &raw)
                            .build();
                        Some(widget.upcast::<gtk::Widget>())
                    }
                    RequestBodyType::File => {
                        let file = body.file().unwrap();
                        let widget = glib::Object::builder::<File>()
                            .property("file", file)
                            .build();

                        // Before the widget can be returned, we need to bind the 'saved' property
                        // first. It is reactive, so that saving a new file for the first time triggers
                        // an update of the property here to enable the buttons.
                        if let Some(pane) = self.grab_base_pane() {
                            // Safety check in case you are trying to use the widget standalone.
                            pane.property_expression("file")
                                .chain_closure::<bool>(glib::closure!(
                                    move |_: glib::Object, file: Option<gtk::gio::File>| file
                                        .is_some()
                                ))
                                .bind(&widget, "saved", Some(&widget));
                        }
                        Some(widget.upcast::<gtk::Widget>())
                    }
                    _ => None,
                };
            self.container.set_child(container_child.as_ref());
        }

        #[template_callback]
        fn on_selection_changed(&self) {
            self.push_body();
            let body = self.obj().body();
            let (body_type, raw_body_type) = cast_selected_entry(self.combo.selected());
            body.set_body_type(body_type);
            if let Some(raw_body_type) = raw_body_type
                && let Some(raw) = body.raw()
            {
                raw.set_payload_type(raw_body_type);
            }
            self.pop_body();
            self.sync_container();
        }

        fn pop_body(&self) {
            let body = self.obj().body();
            match body.body_type() {
                RequestBodyType::UrlEncoded => {
                    let params = self.last_parameters_table.borrow().clone();
                    let urlencoded = body.urlencoded().expect("Urlencoded?");
                    urlencoded.set_params(params);
                }
                RequestBodyType::Multipart => {
                    let params = self.last_parameters_table.borrow().clone();
                    let multipart = body.multipart().expect("Multipart?");
                    multipart.set_params(params);
                }
                RequestBodyType::Raw => {
                    let payload = self.last_raw_payload.borrow().clone();
                    let raw = body.raw().expect("Raw?");
                    raw.set_payload(payload);
                }
                RequestBodyType::File => {
                    let file = self.last_file.borrow().clone();
                    body.set_body_data(file.into());
                }
                _ => {}
            }
        }

        fn push_body(&self) {
            let body = self.obj().body();
            match body.body_type() {
                RequestBodyType::UrlEncoded => {
                    let urlencoded = body.urlencoded().expect("Urlencoded?");
                    self.last_parameters_table.replace(urlencoded.params());
                }
                RequestBodyType::Multipart => {
                    let multipart = body.multipart().expect("Multipart?");
                    self.last_parameters_table.replace(multipart.params());
                }
                RequestBodyType::Raw => {
                    let raw = body.raw().expect("Raw?");
                    self.last_raw_payload.replace(raw.payload());
                }
                RequestBodyType::File => {
                    let file = body.file().expect("File?");
                    self.last_file.replace(file);
                }
                _ => {}
            }
        }

        fn grab_base_pane(&self) -> Option<BasePane> {
            self.obj().ancestor(BasePane::static_type()).and_downcast()
        }
    }

    fn cast_selected_entry(value: u32) -> (RequestBodyType, Option<RequestBodyRawType>) {
        match value {
            1 => (RequestBodyType::UrlEncoded, None),
            2 => (RequestBodyType::Multipart, None),
            3 => (RequestBodyType::Raw, Some(RequestBodyRawType::Json)),
            4 => (RequestBodyType::Raw, Some(RequestBodyRawType::Xml)),
            5 => (RequestBodyType::Raw, Some(RequestBodyRawType::OctetStream)),
            6 => (RequestBodyType::File, None),
            _ => (RequestBodyType::None, None),
        }
    }

    fn cast_entry_to_select(
        auth_type: RequestBodyType,
        auth_body_raw_type: Option<RequestBodyRawType>,
    ) -> u32 {
        match auth_type {
            RequestBodyType::UrlEncoded => 1,
            RequestBodyType::Multipart => 2,
            RequestBodyType::Raw => match auth_body_raw_type.unwrap_or_default() {
                RequestBodyRawType::Json => 3,
                RequestBodyRawType::Xml => 4,
                RequestBodyRawType::OctetStream => 5,
            },
            RequestBodyType::File => 6,
            _ => 0,
        }
    }
}

glib::wrapper! {
    pub struct RequestBodyPane(ObjectSubclass<imp::RequestBodyPane>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
