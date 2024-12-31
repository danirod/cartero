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

use glib::{object::Cast, subclass::types::ObjectSubclassIsExt, Object};
use gtk::{gio, prelude::ListModelExt};

use crate::entities::{KeyValue, KeyValueTable};

use super::KeyValueItem;

mod imp {
    use std::cell::RefCell;

    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use gtk::gio::{self, ListStore};
    use gtk::prelude::ListModelExt;
    use gtk::subclass::prelude::ListModelImpl;

    use crate::objects::KeyValueItem;

    #[derive(Debug)]
    pub struct KeyValueStore {
        pub(super) inner: RefCell<ListStore>,
    }

    impl Default for KeyValueStore {
        fn default() -> Self {
            Self {
                inner: ListStore::with_type(KeyValueItem::static_type()).into(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeyValueStore {
        const NAME: &'static str = "CarteroKeyValueStore";
        type Type = super::KeyValueStore;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for KeyValueStore {
        fn constructed(&self) {
            let inner = self.inner.borrow();
            let obj = &*self.obj();
            inner.connect_items_changed(
                glib::clone!(@weak obj => move |_, position, removed, added| {
                    obj.emit_by_name::<()>("items-changed", &[&position, &removed, &added]);
                }),
            );
        }
    }

    impl ListModelImpl for KeyValueStore {
        fn item_type(&self) -> glib::Type {
            let inner = self.inner.borrow();
            inner.item_type()
        }

        fn n_items(&self) -> u32 {
            let inner = self.inner.borrow();
            inner.n_items()
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            let inner = self.inner.borrow();
            inner.item(position)
        }
    }
}

glib::wrapper! {
    pub struct KeyValueStore(ObjectSubclass<imp::KeyValueStore>)
        @implements gtk::Buildable, gio::ListModel;
}

impl Default for KeyValueStore {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl KeyValueStore {
    pub fn insert(&self, row: &KeyValueItem) {
        let imp = self.imp();
        let store = imp.inner.borrow();
        store.append(row);
    }

    pub fn remove_at(&self, pos: u32) {
        let imp = self.imp();
        let store = imp.inner.borrow();
        store.remove(pos);
    }

    pub fn clear(&self) {
        let imp = self.imp();
        let store = imp.inner.borrow();
        store.remove_all();
    }

    pub fn snapshot(&self) -> KeyValueTable {
        let imp = self.imp();
        let store = imp.inner.borrow();
        let mut table: Vec<KeyValue> = Vec::new();
        for i in 0..store.n_items() {
            let item = store.item(i).unwrap().downcast::<KeyValueItem>().unwrap();
            table.push(KeyValue::from(item));
        }
        KeyValueTable::new(&table)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use glib::{object::Cast, types::StaticType};
    use gtk::prelude::ListModelExt;

    use crate::{entities::KeyValue, objects::KeyValueItem};

    use super::KeyValueStore;

    #[gtk::test]
    pub fn test_create_store_from_default() {
        let store = KeyValueStore::default();
        assert_eq!(0, store.n_items());
        assert_eq!(KeyValueItem::static_type(), store.item_type());
        let kv: KeyValueItem = glib::Object::builder()
            .property("header-name", "Location")
            .property("header-value", "/redirect/to")
            .build();
        store.insert(&kv);
        assert_eq!(1, store.n_items());
    }

    #[gtk::test]
    pub fn test_store_n_items() {
        let kv1: KeyValueItem = glib::Object::builder()
            .property("header-name", "Authorization")
            .property("header-value", "Bearer 12341234")
            .property("secret", true)
            .property("active", true)
            .build();
        let kv2: KeyValueItem = glib::Object::builder()
            .property("header-name", "Location")
            .property("header-value", "/redirect/to")
            .build();
        let store: KeyValueStore = glib::Object::builder().build();
        assert_eq!(0, store.n_items());
        store.insert(&kv1);
        assert_eq!(1, store.n_items());
        store.insert(&kv2);
        assert_eq!(2, store.n_items());
        store.remove_at(0);
        assert_eq!(1, store.n_items());
        store.insert(&kv2);
        assert_eq!(2, store.n_items());
        store.clear();
        assert_eq!(0, store.n_items());
    }

    #[gtk::test]
    pub fn test_store_get_item() {
        let kv1: KeyValueItem = glib::Object::builder()
            .property("header-name", "Authorization")
            .property("header-value", "Bearer 12341234")
            .property("secret", true)
            .property("active", true)
            .build();
        let kv2: KeyValueItem = glib::Object::builder()
            .property("header-name", "Location")
            .property("header-value", "/redirect/to")
            .build();
        let store: KeyValueStore = glib::Object::builder().build();

        assert!(store.item(0).is_none());

        store.insert(&kv1);
        store.insert(&kv2);

        assert!(store.item(0).is_some_and(
            |i| i.downcast::<KeyValueItem>().unwrap().header_name() == "Authorization"
        ));
        assert!(store
            .item(1)
            .is_some_and(|i| i.downcast::<KeyValueItem>().unwrap().header_name() == "Location"));
        assert!(store.item(3).is_none());
    }

    #[gtk::test]
    pub fn test_store_snapshot() {
        let kv1: KeyValueItem = glib::Object::builder()
            .property("header-name", "Authorization")
            .property("header-value", "Bearer 12341234")
            .property("secret", true)
            .property("active", true)
            .build();
        let kv2: KeyValueItem = glib::Object::builder()
            .property("header-name", "Location")
            .property("header-value", "/redirect/to")
            .build();
        let store: KeyValueStore = glib::Object::builder().build();

        {
            let snapshot = store.snapshot();
            assert_eq!(0, snapshot.len());
        }

        store.insert(&kv1);
        store.insert(&kv2);

        {
            let snapshot = store.snapshot();
            assert_eq!(2, snapshot.len());
            assert_eq!(
                KeyValue {
                    name: "Authorization".into(),
                    value: "Bearer 12341234".into(),
                    secret: true,
                    active: true,
                },
                snapshot[0]
            );
            assert_eq!(
                KeyValue {
                    name: "Location".into(),
                    value: "/redirect/to".into(),
                    secret: false,
                    active: true,
                },
                snapshot[1]
            );
        }
    }

    #[gtk::test]
    pub fn test_store_emits_signals() {
        let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        let store: KeyValueStore = glib::Object::builder().build();

        let counter_cb = Arc::clone(&counter);
        store.connect_items_changed(move |_, _, _, _| {
            counter_cb.fetch_add(1, Ordering::Relaxed);
        });

        let kv1: KeyValueItem = glib::Object::builder()
            .property("header-name", "Authorization")
            .property("header-value", "Bearer 12341234")
            .property("secret", true)
            .property("active", true)
            .build();
        let kv2: KeyValueItem = glib::Object::builder()
            .property("header-name", "Location")
            .property("header-value", "/redirect/to")
            .build();

        assert_eq!(counter.load(Ordering::Relaxed), 0);
        store.insert(&kv1);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
        store.insert(&kv2);
        assert_eq!(counter.load(Ordering::Relaxed), 2);
        store.remove_at(0);
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }
}
