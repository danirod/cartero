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

use std::collections::HashMap;

use glib::{
    object::{Cast, CastNone},
    subclass::types::ObjectSubclassIsExt,
};
use gtk::{gio, glib};

use crate::objects::KeyValueItem;

mod imp {
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use std::borrow::BorrowMut;

    use glib::closure_local;
    use glib::subclass::InitializingObject;
    use gtk::gio;
    use gtk::subclass::box_::BoxImpl;
    use gtk::subclass::widget::{CompositeTemplateClass, WidgetImpl};
    use gtk::{glib, CompositeTemplate};

    use crate::objects::KeyValueItem;
    use crate::widgets::KeyValueRow;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/key_value_pane.ui")]
    pub struct KeyValuePane {
        #[template_child]
        list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub selection_model: TemplateChild<gtk::NoSelection>,
        #[template_child]
        add_new: TemplateChild<gtk::Button>,
    }

    #[gtk::template_callbacks]
    impl KeyValuePane {
        #[template_callback]
        fn on_add_new_header(&self) {
            let empty_header = KeyValueItem::default();
            empty_header.set_active(true);
            let store = self
                .selection_model
                .model()
                .and_downcast::<gio::ListStore>()
                .unwrap();
            store.append(&empty_header);
        }

        fn on_remove_row(&self, idx: u32) {
            if let Some(ref mut model) = self.selection_model.model().borrow_mut() {
                let store = model.clone().downcast::<gio::ListStore>().unwrap();
                store.remove(idx);
            };
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeyValuePane {
        const NAME: &'static str = "CarteroKeyValuePane";
        type Type = super::KeyValuePane;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KeyValuePane {
        fn constructed(&self) {
            self.parent_constructed();

            let store = gio::ListStore::new::<KeyValueItem>();
            self.selection_model.set_model(Some(&store));

            /* Build the factory used to link the headers and the widgets. */
            let factory = gtk::SignalListItemFactory::new();
            self.list_view.set_factory(Some(&factory));

            /* Called whenever the system wants a new empty widget. */
            factory.connect_setup(|_, item: &gtk::ListItem| {
                let row = KeyValueRow::default();
                item.set_child(Some(&row));
            });

            /* Called whenever the system wants to stop using a widget. */
            factory.connect_teardown(|_, item: &gtk::ListItem| {
                item.set_child(Option::<&gtk::Widget>::None);
            });

            /* Called whenever the system will place a header in a widget. */
            factory.connect_bind(
                glib::clone!(@weak self as pane => move |_, item: &gtk::ListItem| {
                    let widget = item.child().and_downcast::<KeyValueRow>().unwrap();
                    let header = item.item().and_downcast::<KeyValueItem>().unwrap();

                    /* Add the initial data to the header. */
                    widget.set_header_name(header.header_name());
                    widget.set_header_value(header.header_value());
                    widget.set_active(header.active());

                    /* Create some binds to put the data back in this header. */
                    widget.add_binding(widget
                        .bind_property("header-name", &header, "header-name")
                        .bidirectional()
                        .sync_create()
                        .build());
                    widget.add_binding(widget
                        .bind_property("header-value", &header, "header-value")
                        .bidirectional()
                        .sync_create()
                        .build());
                    widget.add_binding(widget
                        .bind_property("active", &header, "active")
                        .bidirectional()
                        .sync_create()
                        .build());

                    let pos = item.position();
                    let delete_closure = widget.connect_closure("delete", false, closure_local!(@strong pane => move |_row: KeyValueRow| {
                        pane.on_remove_row(pos);
                    }));
                    widget.set_delete_closure(delete_closure);
                }),
            );

            /* Called whenever the system will stop using a header in a widget. */
            factory.connect_unbind(|_, item: &gtk::ListItem| {
                let widget = item.child().and_downcast::<KeyValueRow>().unwrap();

                /* Disconnect the binds stored in the header. */
                widget.reset_bindings();

                /* Remove data from this widget to make it clean. */
                widget.set_header_name("");
                widget.set_header_value("");
                widget.set_active(false);

                /* Disconnect the closure. */
                if let Some(closure_id) = widget.delete_closure() {
                    widget.disconnect(closure_id);
                }
            });
        }
    }

    impl WidgetImpl for KeyValuePane {}

    impl BoxImpl for KeyValuePane {}
}

glib::wrapper! {
    pub struct KeyValuePane(ObjectSubclass<imp::KeyValuePane>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable;
}

impl KeyValuePane {
    pub fn get_entries(&self) -> Vec<KeyValueItem> {
        let imp = self.imp();

        let model = imp.selection_model.model().expect("Where is my ListModel?");
        let list_store = model.downcast::<gio::ListStore>().unwrap();

        list_store
            .into_iter()
            .filter_map(|item| item.ok().and_downcast())
            .collect()
    }

    pub fn set_entries(&self, headers: &HashMap<String, String>) {
        let imp = self.imp();

        let model = imp.selection_model.model().expect("Where is my ListModel?");
        let list_store = model.downcast::<gio::ListStore>().unwrap().clone();
        list_store.remove_all();

        for (k, v) in headers {
            let hdr = KeyValueItem::default();
            hdr.set_header_name(k.clone());
            hdr.set_header_value(v.clone());
            hdr.set_active(true);
            list_store.append(&hdr);
        }
    }
}
