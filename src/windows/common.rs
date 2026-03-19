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

use adw::prelude::AdwApplicationWindowExt;
use glib::object::{CastNone, MayDowncastTo};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

/// A trait to be used by shell widgets.
pub trait CommonShell: WidgetImpl
where
    <Self as ObjectSubclass>::Type: IsA<gtk::Widget>,
{
    /// Returns the root() object, already casted to gtk::ApplicationWindow.
    fn get_application_window(&self) -> Option<gtk::ApplicationWindow> {
        self.obj().root().and_downcast()
    }

    /// Checks if Cartero is running in client-side-decoration mode or not.
    /// The difference is that in CSD, the root window is an
    /// AdwApplicationWindow, because the compositor does not draw window
    /// borders.
    fn is_client_side(&self) -> bool {
        self.get_application_window()
            .and_downcast::<adw::ApplicationWindow>()
            .is_some()
    }
}

pub fn get_window_shell<T>(win: &T) -> Option<gtk::Widget>
where
    T: IsA<gtk::Window> + MayDowncastTo<adw::ApplicationWindow>,
{
    match win.downcast_ref::<adw::ApplicationWindow>() {
        Some(adw_win) => adw_win.content(),
        None => win.child(),
    }
}
