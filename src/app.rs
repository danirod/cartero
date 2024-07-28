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
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::gio::{self, ActionEntryBuilder, Settings};
use gtk::prelude::ActionMapExtManual;

use crate::config::{APP_ID, BASE_ID, RESOURCE_PATH};
use crate::win::CarteroWindow;

#[macro_export]
macro_rules! accelerator {
    ($accel:expr) => {
        if cfg!(target_os = "macos") {
            concat!("<Meta>", $accel)
        } else {
            concat!("<Primary>", $accel)
        }
    };
}

mod imp {
    use std::cell::OnceCell;

    use adw::prelude::*;
    use adw::subclass::application::AdwApplicationImpl;
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk::gio::Settings;
    use gtk::subclass::prelude::*;
    use gtk::subclass::{application::GtkApplicationImpl, prelude::ApplicationImpl};

    use super::*;

    #[derive(Default)]
    pub struct CarteroApplication {
        pub(super) settings: OnceCell<Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroApplication {
        const NAME: &'static str = "CarteroApplication";
        type Type = super::CarteroApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for CarteroApplication {}

    impl ApplicationImpl for CarteroApplication {
        fn activate(&self) {
            self.parent_activate();
            let (window, is_new_window) = match self.obj().active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&self.obj()), true),
            };
            glib::spawn_future_local(glib::clone!(@weak window => async move {
                if is_new_window {
                    window.open_last_session().await;
                }
                window.present();
            }));
        }

        fn startup(&self) {
            self.parent_startup();
            gtk::Window::set_default_icon_name(APP_ID);

            let obj = self.obj();
            obj.set_accels_for_action("win.new", &[accelerator!("t")]);
            obj.set_accels_for_action("win.open", &[accelerator!("o")]);
            obj.set_accels_for_action("win.save", &[accelerator!("s")]);
            obj.set_accels_for_action("win.save-as", &[accelerator!("<Shift>s")]);
            obj.set_accels_for_action("win.close", &[accelerator!("w")]);
            obj.set_accels_for_action("win.request", &[accelerator!("Return")]);
            obj.set_accels_for_action("app.quit", &[accelerator!("q")]);
            obj.set_accels_for_action("win.show-help-overlay", &[accelerator!("question")]);
            obj.setup_app_actions();
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);

            let (window, is_new_window) = match self.obj().active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&self.obj()), true),
            };
            let thread_files: Vec<gio::File> = files.to_vec();
            glib::spawn_future_local(async move {
                if is_new_window {
                    window.open_last_session().await;
                }
                for file in thread_files {
                    window.add_endpoint(Some(&file)).await;
                }
                window.present();
            });
        }
    }

    impl GtkApplicationImpl for CarteroApplication {}

    impl AdwApplicationImpl for CarteroApplication {}
}

glib::wrapper! {
    pub struct CarteroApplication(ObjectSubclass<imp::CarteroApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;

}

impl Default for CarteroApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl CarteroApplication {
    pub fn get() -> Self {
        gio::Application::default()
            .and_downcast::<CarteroApplication>()
            .unwrap()
    }

    pub fn new() -> Self {
        Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .property("resource-base-path", RESOURCE_PATH)
            .build()
    }

    pub fn settings(&self) -> &Settings {
        self.imp().settings.get_or_init(|| Settings::new(BASE_ID))
    }

    fn setup_app_actions(&self) {
        let quit = ActionEntryBuilder::new("quit")
            .activate(glib::clone!(@weak self as app => move |_, _, _| {
                for window in app.windows() {
                    window.close();
                }

                if app.windows().is_empty() {
                    app.quit();
                }
            }))
            .build();

        self.add_action_entries([quit]);
    }
}
