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
use gtk::{gio::ListStore, prelude::*};

use crate::objects::KeyValueItem;

mod imp {
    use adw::subclass::bin::BinImpl;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use std::cell::{OnceCell, RefCell};

    use glib::subclass::InitializingObject;
    use glib::{closure_local, Properties};
    use gtk::gio::ListStore;
    use gtk::subclass::widget::{CompositeTemplateClass, WidgetImpl};
    use gtk::{glib, CompositeTemplate};

    use crate::objects::KeyValueItem;
    use crate::widgets::KeyValueRow;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::KeyValuePane)]
    #[template(resource = "/es/danirod/Cartero/key_value_pane.ui")]
    pub struct KeyValuePane {
        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,

        #[property(get, set = Self::set_model)]
        model: OnceCell<ListStore>,

        #[property(get)]
        valid: RefCell<bool>,
    }

    #[gtk::template_callbacks]
    impl KeyValuePane {
        fn set_model(&self, model: &ListStore) {
            let this_model = self.model.get().unwrap();
            let items = Vec::from_iter(model.iter::<glib::Object>().map(Result::unwrap));
            this_model.remove_all();
            this_model.splice(0, 0, &items);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeyValuePane {
        const NAME: &'static str = "CarteroKeyValuePane";
        type Type = super::KeyValuePane;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for KeyValuePane {
        fn constructed(&self) {
            self.parent_constructed();

            self.model
                .set(ListStore::with_type(KeyValueItem::static_type()))
                .unwrap();
            self.list_box.bind_model(self.model.get(),
            glib::clone!(@weak self as pane => @default-panic, move |item| {
                let item = item.downcast_ref::<KeyValueItem>().unwrap();
                let row = KeyValueRow::default();
                row.add_binding(item.bind_property("header-name", &row, "header-name")
                    .bidirectional()
                    .sync_create()
                    .build());
                row.add_binding(item.bind_property("header-value", &row, "header-value")
                    .bidirectional()
                    .sync_create()
                    .build());
                row.add_binding(item.bind_property("active", &row, "active")
                    .bidirectional()
                    .sync_create()
                    .build());
                row.add_binding(item.bind_property("secret", &row, "secret")
                    .bidirectional()
                    .sync_create()
                    .build());
                let pane_delete = pane.clone();
                row.connect_closure("delete", false, closure_local!(@strong item => move |_: KeyValueRow| {
                    let model = pane_delete.model.get().unwrap();
                    if let Some(pos) = model.find(&item) {
                        model.remove(pos);

                        let obj = pane_delete.obj();
                        obj.assert_always_placeholder();
                    }
                }));

                let pane_changed = pane.clone();
                item.connect_closure("changed", false, closure_local!(move |_: KeyValueItem| {
                    let obj = pane_changed.obj();
                    obj.assert_always_placeholder();
                }));
                row.upcast::<gtk::Widget>()
            }));
        }
    }

    impl WidgetImpl for KeyValuePane {}

    impl BinImpl for KeyValuePane {}
}

glib::wrapper! {
    pub struct KeyValuePane(ObjectSubclass<imp::KeyValuePane>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable;
}

impl Default for KeyValuePane {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl KeyValuePane {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assert_always_placeholder(&self) {
        let model = &self.model();
        let empty = model.iter::<KeyValueItem>().any(|row| {
            let Ok(row) = row else {
                return false;
            };
            row.header_name().is_empty() && row.header_value().is_empty()
        });
        if !empty {
            let new_row = KeyValueItem::new();
            model.append(&new_row);
        }
    }

    pub fn get_entries(&self) -> Vec<KeyValueItem> {
        let model = &self.model();
        let iter = model.iter::<KeyValueItem>();
        iter.filter(|value| value.is_ok()).flatten().collect()
    }

    pub fn set_entries(&self, entries: &[KeyValueItem]) {
        let store = ListStore::with_type(KeyValueItem::static_type());
        store.extend_from_slice(entries);
        self.set_model(&store);
        self.assert_always_placeholder();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gtk::gio::{prelude::*, ListStore};

    use super::*;

    #[gtk::test]
    pub fn test_append_to_model() {
        crate::init_test_resources();

        let key_value_pane = KeyValuePane::default();
        let model = &key_value_pane.model();
        assert_eq!(model.n_items(), 0);

        let accept = KeyValueItem::new_with_value("Accept", "application/html");
        model.append(&accept);
        assert_eq!(model.n_items(), 1);
    }

    #[gtk::test]
    pub fn test_set_model() {
        crate::init_test_resources();

        let ctype = KeyValueItem::new_with_value("Content-Type", "application/json");
        let clen = KeyValueItem::new_with_value("Content-Length", "42");
        let list = ListStore::with_type(KeyValueItem::static_type());
        list.append(&ctype);
        list.append(&clen);
        assert_eq!(list.n_items(), 2);

        let pane = KeyValuePane::default();
        pane.set_model(&list);
        assert_eq!(pane.model().n_items(), 2);
    }

    #[gtk::test]
    pub fn test_set_model_overrides() {
        crate::init_test_resources();

        let key_value_pane = KeyValuePane::default();
        let accept = KeyValueItem::new_with_value("Accept", "application/html");
        key_value_pane.model().append(&accept);

        let keys: Vec<String> = key_value_pane
            .model()
            .iter::<KeyValueItem>()
            .map(|obj| obj.unwrap().header_name())
            .collect();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "Accept");

        let ctype = KeyValueItem::new_with_value("Content-Type", "application/json");
        let clen = KeyValueItem::new_with_value("Content-Length", "42");
        let list = ListStore::with_type(KeyValueItem::static_type());
        list.append(&ctype);
        list.append(&clen);
        key_value_pane.set_model(&list);

        let keys: Vec<String> = key_value_pane
            .model()
            .iter::<KeyValueItem>()
            .map(|obj| obj.unwrap().header_name())
            .collect();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], "Content-Type");
        assert_eq!(keys[1], "Content-Length");
    }

    #[gtk::test]
    pub fn test_set_model_emits_signal() {
        crate::init_test_resources();

        let ctype = KeyValueItem::new_with_value("Content-Type", "application/json");
        let clen = KeyValueItem::new_with_value("Content-Length", "42");
        let list = ListStore::with_type(KeyValueItem::static_type());
        list.append(&ctype);
        list.append(&clen);
        assert_eq!(list.n_items(), 2);

        let connected = Rc::new(Cell::new(false));

        let pane = KeyValuePane::default();
        pane.model()
            .connect_items_changed(glib::clone!(@strong connected => move |_, _, _, _| {
                connected.set(true);
            }));
        pane.set_model(&list);
        assert!(connected.get());
    }

    #[gtk::test]
    pub fn test_model_get_set_entries() {
        let ctype = KeyValueItem::new_with_value("Content-Type", "application/json");
        let clen = KeyValueItem::new_with_value("Content-Length", "42");
        let slice = &[ctype, clen];

        let pane = KeyValuePane::default();
        pane.set_entries(slice);

        let model = pane.model();
        assert_eq!(model.n_items(), 2);
        let keys: Vec<String> = pane
            .model()
            .iter::<KeyValueItem>()
            .map(|obj| obj.unwrap().header_name())
            .collect();
        assert_eq!("Content-Type", keys[0]);
        assert_eq!("Content-Length", keys[1]);

        let entries = pane.get_entries();
        assert_eq!(2, entries.len());
        assert_eq!("Content-Type", entries[0].header_name());
        assert_eq!("Content-Length", entries[1].header_name());
    }
}
