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

use gettextrs::gettext;
use gtk::gio::ListStore;

const LOCALES: [(&str, &str); 13] = [
    ("ca", "Català"),
    ("cs", "Čeština"),
    ("de", "Deutsch"),
    ("eo", "Esperanto"),
    ("es", "Español"),
    ("eu", "Euskera"),
    ("fr", "Français"),
    ("gl", "Galego"),
    ("pt", "Português"),
    ("pt_BR", "Português (Brasil)"),
    ("ro", "Română"),
    ("ru", "Русский язык"),
    ("ta", "தமிழ்"),
];

use glib::subclass::prelude::*;
use glib::{prelude::*, Object};

mod imp {
    use std::cell::RefCell;

    use glib::Properties;

    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type = super::LocaleRepr)]
    pub struct LocaleRepr {
        #[property(get, construct_only)]
        iso: RefCell<String>,

        #[property(get, construct_only)]
        name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocaleRepr {
        const NAME: &'static str = "CarteroLocaleRepr";
        type Type = super::LocaleRepr;
    }

    #[glib::derived_properties]
    impl ObjectImpl for LocaleRepr {}
}

glib::wrapper! {
    pub struct LocaleRepr(ObjectSubclass<imp::LocaleRepr>);
}

impl LocaleRepr {
    pub fn get_model() -> ListStore {
        let store = ListStore::new::<Self>();
        let default: Self = Object::builder()
            .property("iso", "")
            .property("name", &gettext("Follow system settings"))
            .build();
        store.append(&default);
        for (iso, name) in LOCALES {
            let repr: Self = Object::builder()
                .property("iso", iso.to_string())
                .property("name", name.to_string())
                .build();
            store.append(&repr);
        }
        store
    }

    pub fn index_for_code(code: &str) -> Option<u32> {
        if code.is_empty() {
            return Some(0);
        }
        LOCALES
            .iter()
            .enumerate()
            .find(|(_, (iso, _))| *iso == code)
            .map(|(idx, _)| (idx as u32) + 1)
    }

    pub fn index_to_code(idx: u32) -> Option<&'static str> {
        if idx == 0 {
            return Some("");
        }
        let array_idx = (idx as usize) - 1;
        if array_idx < LOCALES.len() {
            Some(LOCALES[array_idx].0)
        } else {
            None
        }
    }
}
