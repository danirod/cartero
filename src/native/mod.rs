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

use gtk::prelude::WidgetExt;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum ColorScheme {
    Light,
    Dark,
    System,
}

impl From<adw::ColorScheme> for ColorScheme {
    fn from(value: adw::ColorScheme) -> Self {
        match value {
            adw::ColorScheme::ForceDark | adw::ColorScheme::PreferDark => Self::Dark,
            adw::ColorScheme::ForceLight | adw::ColorScheme::PreferLight => Self::Light,
            _ => Self::System,
        }
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum VibrancyMode {
    None,
    Standard,
    Sidebar,
}

pub(crate) fn set_window_theme(win: &gtk::Window, color_scheme: adw::ColorScheme) {
    let scheme = ColorScheme::from(color_scheme);
    if cfg!(target_os = "macos") {
        self::macos::set_window_theme(win, scheme);
    }
}

pub(crate) fn update_vibrancy(win: &gtk::Window, headerbar_height: Option<i32>, sidebar_width: Option<i32>) {
    if cfg!(target_os = "macos") {
        self::macos::update_vibrancy(win, sidebar_width, headerbar_height);
    }
}

/// Initialises the native elements for the window. This function does not initialise the
/// vibrancy, because it has to receive the window metrics to do so, and they will change
/// every time the window is resized, so remember to call update_vibrancy() if you want
/// some of that.
pub(crate) fn prepare_window(win: &gtk::Window) {
    // Marker class.
    if cfg!(windows) {
        win.add_css_class("win32-native");
    } else if cfg!(target_os = "macos") {
        win.add_css_class("macos-native");
    }

    win.connect_realize(|win| {
        let style_manager = adw::StyleManager::default();
        let color_scheme = style_manager.color_scheme();
        set_window_theme(&win, color_scheme);
    });

    let style_manager = adw::StyleManager::default();
    style_manager.connect_color_scheme_notify(glib::clone!(#[weak] win, move |sm| {
        set_window_theme(&win, sm.color_scheme());
    }));
    style_manager.connect_dark_notify(glib::clone!(#[weak] win, move |sm| {
        set_window_theme(&win, sm.color_scheme());
    }));
    style_manager.connect_high_contrast_notify(glib::clone!(#[weak] win, move |sm| {
        set_window_theme(&win, sm.color_scheme());
    }));
}