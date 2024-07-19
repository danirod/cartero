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

use std::path::{Path, PathBuf};

use adw::prelude::*;
use gettextrs::gettext;
use glib::Object;

use crate::error::CarteroError;

use super::EndpointPane;

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::Properties;
    use gtk::glib;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ItemPane)]
    pub struct ItemPane {
        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set, nullable)]
        path: RefCell<Option<String>>,

        #[property(get, set)]
        pub dirty: RefCell<bool>,

        #[property(get = Self::window_title)]
        pub _window_title: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemPane {
        const NAME: &'static str = "CarteroItemPane";
        type Type = super::ItemPane;
        type ParentType = adw::Bin;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ItemPane {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.connect_dirty_notify(glib::clone!(@weak obj as window => move |_| {
                window.notify_window_title();
            }));
            obj.connect_title_notify(glib::clone!(@weak obj as window => move |_| {
                window.notify_window_title();
            }));
        }
    }

    impl WidgetImpl for ItemPane {}

    impl BinImpl for ItemPane {}

    impl ItemPane {
        fn window_title(&self) -> String {
            if *self.dirty.borrow() {
                format!("• {}", *self.title.borrow())
            } else {
                (*self.title.borrow()).clone()
            }
        }
    }
}

glib::wrapper! {
    pub struct ItemPane(ObjectSubclass<imp::ItemPane>)
        @extends gtk::Widget, adw::Bin;
}

impl ItemPane {
    pub fn new_for_endpoint(path: Option<&PathBuf>) -> Result<Self, CarteroError> {
        let pane: Self = match path {
            Some(path) => {
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                Object::builder()
                    .property("title", file_name)
                    .property("path", Some(path.clone()))
                    .build()
            }
            None => Object::builder()
                .property("title", gettext("(untitled)"))
                .build(),
        };

        let child_pane = EndpointPane::default();
        pane.set_child(Some(&child_pane));
        child_pane.set_item_pane(Some(&pane));

        if let Some(path) = path {
            let contents = crate::file::read_file(path)?;
            let endpoint = crate::file::parse_toml(&contents)?;
            child_pane.assign_endpoint(&endpoint);
        }

        Ok(pane)
    }

    pub fn endpoint(&self) -> Option<EndpointPane> {
        self.child().and_downcast::<EndpointPane>()
    }

    pub fn update_title_and_path(&self, path: &Path) {
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        self.set_title(file_name);
        self.set_path(Some(path.to_str().unwrap()));
    }
}
