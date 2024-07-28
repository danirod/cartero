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

use adw::prelude::*;
use gettextrs::gettext;
use glib::Object;
use gtk::{gio, ClosureExpression};

use crate::error::CarteroError;

use super::EndpointPane;

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::Properties;
    use gtk::{gio, glib};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ItemPane)]
    pub struct ItemPane {
        #[property(get, set, nullable)]
        file: RefCell<Option<gio::File>>,

        #[property(get, set)]
        pub dirty: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemPane {
        const NAME: &'static str = "CarteroItemPane";
        type Type = super::ItemPane;
        type ParentType = adw::Bin;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ItemPane {}

    impl WidgetImpl for ItemPane {}

    impl BinImpl for ItemPane {}

    impl ItemPane {}
}

glib::wrapper! {
    pub struct ItemPane(ObjectSubclass<imp::ItemPane>)
        @extends gtk::Widget, adw::Bin;
}

impl ItemPane {
    pub async fn new_for_endpoint(file: Option<&gio::File>) -> Result<Self, CarteroError> {
        let pane: Self = Object::builder().property("file", file).build();

        let child_pane = EndpointPane::default();
        pane.set_child(Some(&child_pane));

        if let Some(path) = file {
            let contents = crate::file::read_file(path).await?;
            let endpoint = crate::file::parse_toml(&contents)?;
            child_pane.assign_endpoint(&endpoint);
        }

        child_pane.set_item_pane(Some(&pane));

        Ok(pane)
    }

    pub fn endpoint(&self) -> Option<EndpointPane> {
        self.child().and_downcast::<EndpointPane>()
    }

    pub fn window_title_binding(&self) -> ClosureExpression {
        ClosureExpression::new::<String>(
            [
                &self.property_expression("file"),
                &self.property_expression("dirty"),
            ],
            glib::closure!(|_: ItemPane, file: Option<gio::File>, dirty: bool| {
                let title = file
                    .and_then(|f| f.basename())
                    .map(|bn| bn.file_stem().unwrap().to_str().unwrap().to_string())
                    .unwrap_or(gettext("(untitled)"));
                if dirty {
                    format!("• {}", &title)
                } else {
                    title
                }
            }),
        )
    }

    pub fn window_subtitle_binding(&self) -> ClosureExpression {
        ClosureExpression::new::<String>(
            [&self.property_expression("file")],
            glib::closure!(|_: ItemPane, file: Option<gio::File>| {
                file.and_then(|f| f.path())
                    .map(|bn| bn.display().to_string())
                    .unwrap_or(gettext("Draft"))
            }),
        )
    }
}
