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

use glib::Object;
use gtk4::gio;

use crate::config::APP_ID;
use crate::win::CarteroWindow;

mod imp {
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::subclass::{application::GtkApplicationImpl, prelude::ApplicationImpl};

    use super::*;

    #[derive(Default)]
    pub struct CarteroApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroApplication {
        const NAME: &'static str = "CarteroApplication";
        type Type = super::CarteroApplication;
        type ParentType = gtk4::Application;
    }

    impl ObjectImpl for CarteroApplication {}

    impl ApplicationImpl for CarteroApplication {
        fn activate(&self) {
            self.parent_activate();
            self.obj().get_window().present();
        }

        fn startup(&self) {
            self.parent_startup();
            gtk4::Window::set_default_icon_name(APP_ID);
        }
    }

    impl GtkApplicationImpl for CarteroApplication {}
}

glib::wrapper! {
    pub struct CarteroApplication(ObjectSubclass<imp::CarteroApplication>)
        @extends gio::Application, gtk4::Application,
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
        CarteroWindow::new(self)
    }
}
