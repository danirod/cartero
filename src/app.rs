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
use gtk::gio::{self, Settings};

use crate::{
    config::{APP_ID, BASE_ID, RESOURCE_PATH},
    widgets::standalone::Shell,
};

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
    use gtk::gio::{ActionEntry, Settings};
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
            self.obj().new_window();
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
            obj.setup_color_scheme();

            self.init_actions();
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);
        }
    }

    impl GtkApplicationImpl for CarteroApplication {}

    impl AdwApplicationImpl for CarteroApplication {}

    impl CarteroApplication {
        fn init_actions(&self) {
            let new_window = ActionEntry::builder("new-window")
                .activate(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _, _| {
                        imp.obj().new_window();
                    }
                ))
                .build();
            self.obj().add_action_entries([new_window]);
        }
    }
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

    pub fn new_window(&self) -> Shell {
        let shell = Shell::default();

        let use_csd = true;
        let window: gtk::ApplicationWindow = if use_csd {
            let window = adw::ApplicationWindow::new(self);
            window.set_content(Some(&shell));
            window.upcast()
        } else {
            let window = gtk::ApplicationWindow::new(self);
            window.set_child(Some(&shell));
            window.upcast()
        };

        //let group = gtk::WindowGroup::new();
        //group.add_window(&window);

        window.set_default_size(800, 500);
        window.present();

        shell
    }
}
