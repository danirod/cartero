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

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[derive(Eq, PartialEq)]
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

pub(crate) fn set_window_theme(win: &gtk::Window, color_scheme: adw::ColorScheme) {
    let scheme = ColorScheme::from(color_scheme);
    if cfg!(target_os = "macos") {
        self::macos::set_window_theme(win, scheme);
    }
}

pub(crate) fn prepare_window(win: &gtk::Window) {
    let style_manager = adw::StyleManager::default();
    let color_scheme = style_manager.color_scheme();
    set_window_theme(win, color_scheme);

    style_manager.connect_color_scheme_notify(glib::clone!(#[weak] win, move |sm| {
        set_window_theme(&win, sm.color_scheme());
    }));
    style_manager.connect_high_contrast_notify(glib::clone!(#[weak] win, move |sm| {
        set_window_theme(&win, sm.color_scheme());
    }));
}