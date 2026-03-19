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

use gio::prelude::FileExt;
use glib::prelude::*;
use glib::subclass::prelude::*;

use crate::FieldTable;

glib::wrapper! {
    pub struct EnvFile(ObjectSubclass<imp::EnvFile>) @implements gio::ListModel;
}

impl Default for EnvFile {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EnvFile {
    pub fn builder() -> builder::EnvFileBuilder {
        builder::EnvFileBuilder::new()
    }

    pub fn field_table(&self) -> FieldTable {
        let ft = FieldTable::default();
        ft.replace(&self.imp().env_vars.borrow());
        ft
    }

    pub fn locate_for_path(file: &gio::File) -> Option<gio::File> {
        match file.parent() {
            Some(parent) => {
                let env = parent.child(".env");
                if env.query_exists(gio::Cancellable::NONE) {
                    Some(env)
                } else {
                    Self::locate_for_path(&parent)
                }
            }
            _ => None,
        }
    }
}

mod builder {
    use glib::object::ObjectBuilder;

    use super::*;

    pub struct EnvFileBuilder {
        builder: ObjectBuilder<'static, EnvFile>,
    }

    impl Default for EnvFileBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl EnvFileBuilder {
        pub fn new() -> Self {
            Self {
                builder: glib::Object::builder(),
            }
        }

        pub fn file(mut self, file: Option<&gio::File>) -> Self {
            self.builder = self.builder.property("file", file);
            self
        }

        pub fn build(self) -> EnvFile {
            self.builder.build()
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use gio::{
        prelude::{FileExt, FileMonitorExt, ListModelExt},
        subclass::prelude::ListModelImpl,
    };
    use glib::{Properties, SignalHandlerId, property::PropertySet};

    use crate::{Field, FieldTable};

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::EnvFile)]
    pub struct EnvFile {
        #[property(get, set, nullable)]
        file: RefCell<Option<gio::File>>,

        monitor: RefCell<Option<gio::FileMonitor>>,
        monitor_handler: RefCell<Option<SignalHandlerId>>,
        pub(super) env_vars: RefCell<FieldTable>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EnvFile {
        const NAME: &'static str = "CarteroEnvFile";
        type Type = super::EnvFile;
        type Interfaces = (gio::ListModel,);
    }

    #[glib::derived_properties]
    impl ObjectImpl for EnvFile {
        fn constructed(&self) {
            self.set_monitor();
            self.obj().connect_file_notify(|env_file| {
                env_file.imp().set_monitor();
            });
        }
    }

    impl ListModelImpl for EnvFile {
        fn item(&self, position: u32) -> Option<glib::Object> {
            self.env_vars.borrow().item(position)
        }

        fn item_type(&self) -> glib::Type {
            Field::static_type()
        }

        fn n_items(&self) -> u32 {
            self.env_vars.borrow().n_items()
        }
    }

    impl EnvFile {
        fn set_monitor(&self) {
            // Disconnect the old monitor if one is present.
            if let Some(old_monitor_handler) = self.monitor_handler.replace(None)
                && let Some(old_monitor) = &*self.monitor.borrow()
            {
                old_monitor.disconnect(old_monitor_handler);
            }

            let next_monitor = match self.obj().file() {
                Some(file) => {
                    self.reload_env(&file);
                    let monitor = file
                        .monitor_file(gio::FileMonitorFlags::empty(), gio::Cancellable::NONE)
                        .ok();
                    if let Some(monitor) = &monitor {
                        monitor.connect_changed(glib::clone!(
                            #[weak(rename_to = imp)]
                            self,
                            move |_, file, _, event_type| {
                                dbg!(event_type);
                                imp.reload_env(file);
                            }
                        ));
                    }
                    monitor
                }
                None => {
                    self.clean_env();
                    None
                }
            };
            self.monitor.set(next_monitor);
        }

        fn reload_env(&self, file: &gio::File) {
            let old_env_vars = self.env_vars.borrow().n_items();
            self.env_vars.borrow().clear();

            let path = file.path().expect("dotenv file has no path?");
            match dotenvy::from_path_iter(&path) {
                Ok(iter) => {
                    for var in iter {
                        match var {
                            Ok((key, value)) => {
                                let field = Field::builder().key(key).value(value).build();
                                self.env_vars.borrow().insert(&field);
                            }
                            Err(e) => {
                                println!("ERROR {e:?}");
                                glib::g_warning!(
                                    "es.danirod.Cartero",
                                    "Invalid read for .env file, ignoring! {e:?}",
                                )
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("ERROR {e:?}");

                    glib::g_warning!(
                        "es.danirod.Cartero",
                        "Invalid read for .env file, ignoring! {e:?}"
                    )
                }
            }

            let new_env_vars = self.env_vars.borrow().n_items();
            self.obj().items_changed(0, old_env_vars, new_env_vars);
        }

        fn clean_env(&self) {
            let old_env_vars = self.env_vars.borrow().n_items();
            self.env_vars.borrow().clear();
            self.obj().items_changed(0, old_env_vars, 0);
        }
    }
}
