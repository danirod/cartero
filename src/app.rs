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
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::gio::{self, ActionEntryBuilder, Settings};
use gtk::prelude::ActionMapExtManual;

use crate::config::{APP_ID, BASE_ID, RESOURCE_PATH};
use crate::win::CarteroWindow;
use crate::windows::SettingsDialog;

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
            let obj = self.obj();

            let (window, is_new_window) = match obj.active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&obj), true),
            };

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if is_new_window {
                        let last_session = obj.last_session_tabs();
                        let open_result = window.open_endpoints(&last_session).await;
                        window.present();
                        window.report_open_endpoints_errors(&open_result).await;
                    } else {
                        window.present();
                    }
                }
            ));
        }

        fn startup(&self) {
            self.parent_startup();
            gtk::Window::set_default_icon_name(APP_ID);

            if cfg!(target_os = "windows") {
                if let Some(settings) = gtk::Settings::default() {
                    settings.set_gtk_font_name(Some("Segoe UI 10"));
                }
            } else if cfg!(target_os = "macos") {
                if let Some(settings) = gtk::Settings::default() {
                    settings.set_gtk_font_name(Some(".AppleSystemUIFont 14.5"));
                }
            }

            let obj = self.obj();
            obj.set_accels_for_action("win.new", &[accelerator!("t")]);
            obj.set_accels_for_action("win.open", &[accelerator!("o")]);
            obj.set_accels_for_action("win.save", &[accelerator!("s")]);
            obj.set_accels_for_action("win.save-as", &[accelerator!("<Shift>s")]);
            obj.set_accels_for_action("win.close", &[accelerator!("w")]);
            obj.set_accels_for_action("win.request", &[accelerator!("Return")]);
            obj.set_accels_for_action("app.preferences", &[accelerator!("comma")]);
            obj.set_accels_for_action("app.quit", &[accelerator!("q")]);
            obj.set_accels_for_action("win.show-help-overlay", &[accelerator!("question")]);
            obj.setup_app_actions();
            obj.setup_color_scheme();
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);
            let obj = self.obj();
            let (window, is_new_window) = match self.obj().active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&self.obj()), true),
            };

            /* If it's a new window, also open the files from the previous session. */
            let files_to_open: Vec<gio::File> = if is_new_window {
                let mut previous_files = obj.last_session_tabs();
                previous_files.extend(files.to_vec());
                previous_files
            } else {
                files.to_vec()
            };

            glib::spawn_future_local(async move {
                let open_result = window.open_endpoints(&files_to_open).await;
                window.present();
                window.report_open_endpoints_errors(&open_result).await;
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

    fn setup_color_scheme(&self) {
        let settings = self.settings();
        settings
            .bind("application-theme", &self.style_manager(), "color-scheme")
            .mapping(|val, _| {
                let scheme = match val.get::<String>().unwrap().as_str() {
                    "light" => adw::ColorScheme::ForceLight,
                    "dark" => adw::ColorScheme::ForceDark,
                    _ => adw::ColorScheme::Default,
                };
                Some(scheme.into())
            })
            .build();
    }

    fn setup_app_actions(&self) {
        let settings = ActionEntryBuilder::new("preferences")
            .activate(glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _, _| {
                    if let Some(window) = app.active_window() {
                        SettingsDialog::present_for_window(&window);
                    }
                }
            ))
            .build();
        let quit = ActionEntryBuilder::new("quit")
            .activate(glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _, _| {
                    for window in app.windows() {
                        window.close();
                    }

                    if app.windows().is_empty() {
                        app.quit();
                    }
                }
            ))
            .build();

        self.add_action_entries([settings, quit]);
    }

    pub fn last_session_tabs(&self) -> Vec<gio::File> {
        let settings = self.settings();

        settings
            .get::<Vec<String>>("open-files")
            .iter()
            .filter_map(|path| {
                // These paths are in the format file:///home/user/endpoint.cartero.
                // TODO: Can't use the url crate to extract the path from the scheme?
                if let Some((_type, path)) = path.split_once(':') {
                    let file = gio::File::for_path(path);
                    Some(file)
                } else {
                    None
                }
            })
            .collect()
    }
}
