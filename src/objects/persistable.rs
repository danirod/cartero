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

use glib::object::IsA;

use crate::file::FileError;

mod imp {
    use std::sync::OnceLock;

    use glib::{subclass::prelude::*, ParamSpecBuilderExt};

    // FIXME: This syntax to be rewritten in glib-rs 0.20 format

    #[allow(dead_code)]
    #[derive(Copy, Clone, Debug)]
    pub struct Persistable(glib::gobject_ffi::GTypeInterface);

    #[glib::object_interface]
    unsafe impl ObjectInterface for Persistable {
        const NAME: &'static str = "CarteroPersistable";
        type Prerequisites = ();

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("path")
                        .default_value(None)
                        .readwrite()
                        .build(),
                    glib::ParamSpecBoolean::builder("dirty")
                        .default_value(false)
                        .readwrite()
                        .build(),
                ]
            })
        }
    }
}

glib::wrapper! {
    pub struct Persistable(ObjectInterface<imp::Persistable>);
}

pub trait PersistableExt: IsA<Persistable> {
    async fn load_file(&self) -> Result<(), FileError>;

    async fn save_file(&self) -> Result<(), FileError>;
}
