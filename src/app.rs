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

use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::gio::{self, Settings};

use crate::config::APP_ID;
use crate::win::CarteroWindow;

mod imp {
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk::gdk::Display;
    use gtk::gio::Settings;
    use gtk::subclass::prelude::*;
    use gtk::subclass::{application::GtkApplicationImpl, prelude::ApplicationImpl};
    use gtk::{prelude::*, CssProvider};

    use super::*;

    pub struct CarteroApplication {
        pub(super) settings: Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroApplication {
        const NAME: &'static str = "CarteroApplication";
        type Type = super::CarteroApplication;
        type ParentType = gtk::Application;

        fn new() -> Self {
            Self {
                settings: Settings::new(APP_ID),
            }
        }
    }

    impl ObjectImpl for CarteroApplication {}

    impl ApplicationImpl for CarteroApplication {
        fn activate(&self) {
            self.parent_activate();
            self.obj().get_window().present();
        }

        fn startup(&self) {
            self.parent_startup();
            gtk::Window::set_default_icon_name(APP_ID);

            let obj = self.obj();
            obj.set_accels_for_action("win.request", &["<Primary>Return"]);

            let provider = CssProvider::new();
            provider.load_from_resource("/es/danirod/Cartero/style.css");
            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Could not connect to a display."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    impl GtkApplicationImpl for CarteroApplication {}
}

glib::wrapper! {
    pub struct CarteroApplication(ObjectSubclass<imp::CarteroApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionMap, gio::ActionGroup;

}

impl Default for CarteroApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl CarteroApplication {
    pub fn new() -> Self {
        Object::builder().property("application-id", APP_ID).build()
    }

    pub fn get_window(&self) -> CarteroWindow {
        let win = CarteroWindow::new(self);
        win.assign_settings(self.settings());
        win
    }

    pub fn settings(&self) -> &Settings {
        &self.imp().settings
    }
}
