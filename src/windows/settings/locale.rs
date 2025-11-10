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

const LOCALES: [(&str, &str); 15] = [
    ("ca", "Català"),
    ("cs", "Čeština"),
    ("de", "Deutsch"),
    ("eo", "Esperanto"),
    ("es", "Español"),
    ("eu", "Euskera"),
    ("fr", "Français"),
    ("gl", "Galego"),
    ("id", "Bahasa Indonesia"),
    ("pt", "Português"),
    ("pt_BR", "Português (Brasil)"),
    ("ro", "Română"),
    ("ru", "Русский язык"),
    ("ta", "தமிழ்"),
    ("uk", "Українська"),
];

use glib::subclass::prelude::*;
use glib::{prelude::*, Object};

// Returns true if there is a file called locale/{iso}/LC_MESSAGES/cartero.mo in the datadir.
fn locale_exists(iso: &str) -> bool {
    let path = format!("share/locale/{iso}/LC_MESSAGES/cartero.mo");
    let file = crate::app_rel_path(&path);
    file.exists() && file.is_file()
}

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
        // Has to be a separate variable due to variable lifetimes.
        let system = gettext("Follow system settings");
        let mut languages = Vec::from(LOCALES);
        languages.push(("", &system));
        languages.push(("en", "English"));
        languages.sort_by_key(|l| l.0);

        let store = ListStore::new::<Self>();
        for (iso, name) in languages {
            if iso == "" || iso == "en" || locale_exists(iso) {
                let repr: Self = Object::builder()
                    .property("iso", iso.to_string())
                    .property("name", name.to_string())
                    .build();
                store.append(&repr);
            }
        }
        store
    }
}
