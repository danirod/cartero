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

use gdk4_macos::MacosSurface;
use glib::object::CastNone;
use gtk::prelude::NativeExt;
use objc2::{class, msg_send, runtime::AnyObject};
use objc2_app_kit::{NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSWindow};

fn ns_window(win: &gtk::Window) -> Option<*mut NSWindow> {
    if let Some(surface) = win.surface().and_downcast::<MacosSurface>() {
        let obj = surface.native();
        let ns_window = obj as *mut NSWindow;
        Some(ns_window)
    } else {
        None
    }
}

pub(super) fn set_window_theme(win: &gtk::Window, color_scheme: super::ColorScheme) {
    if let Some(ns_window) = ns_window(win) {
        unsafe {
            let appearance = match color_scheme {
                super::ColorScheme::System => std::ptr::null::<AnyObject>(),
                other => {
                    let name = if other == super::ColorScheme::Dark {
                        NSAppearanceNameDarkAqua
                    } else {
                        NSAppearanceNameAqua
                    };
                    msg_send![class!(NSAppearance), appearanceNamed: &*name]
                },
            };
            let ns_window = ns_window as *mut AnyObject;
            let _: () = msg_send![ns_window, setAppearance: appearance];
        }
    }
}