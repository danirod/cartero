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

use adw::prelude::*;
use gettextrs::gettext;
use glib::Object;
use gtk::{gio, ClosureExpression};

use crate::{
    app::CarteroApplication,
    error::FileOperationError,
    objects::{Serializable, SerializableExt},
};

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
    pub fn serializable(&self) -> Option<Serializable> {
        self.child().and_dynamic_cast::<Serializable>().ok()
    }

    pub fn new_for_endpoint(file: Option<&gio::File>) -> Self {
        let endpoint = EndpointPane::default();
        let pane: Self = Object::builder()
            .property("file", file)
            .property("child", Some(&endpoint))
            .build();
        endpoint
            .bind_property("dirty", &pane, "dirty")
            .bidirectional()
            .sync_create()
            .build();
        pane
    }

    pub fn endpoint(&self) -> Option<EndpointPane> {
        self.child().and_downcast::<EndpointPane>()
    }

    pub async fn load_pane(&self) -> Result<(), FileOperationError> {
        if let Some(serial) = self.serializable() {
            let Some(file) = self.file() else {
                return Err(FileOperationError::NoFileGiven);
            };
            let contents = file
                .load_contents_future()
                .await
                .map(|slice| String::from_utf8_lossy(&slice.0).to_string())
                .map_err(|_| FileOperationError::FileReadError)?;
            serial
                .from_toml(&contents)
                .map_err(|_| FileOperationError::FileDecodeError)?;
            serial.set_dirty(false);
        }
        Ok(())
    }

    pub async fn save_pane(&self) -> Result<(), FileOperationError> {
        let make_backup = {
            let app = CarteroApplication::default();
            let settings = app.settings();
            settings.get::<bool>("create-backup-files")
        };
        if let Some(serial) = self.serializable() {
            let Some(file) = self.file() else {
                return Err(FileOperationError::NoFileGiven);
            };
            let toml = serial
                .to_toml()
                .map_err(|_| FileOperationError::FileEncodeError)?;
            file.replace_contents_future(toml, None, make_backup, gio::FileCreateFlags::NONE)
                .await
                .map_err(|_| FileOperationError::FileWriteError)?;
        }
        Ok(())
    }

    pub fn clear_dirty(&self) {
        if let Some(serializable) = self.serializable() {
            serializable.set_dirty(false);
        }
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
