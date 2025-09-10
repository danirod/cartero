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

use gdk4_win32::Win32Surface;
use gtk::prelude::*;

mod helpers;

// TODO: Can I do all the things I do with winapi and winreg, with windows-sys
// instead, so that I can use one crate instead of three for the same things?

use winapi::shared::minwindef::BOOL;
use winapi::{shared::windef::HWND, um::dwmapi::DwmSetWindowAttribute};

use crate::native::{ColorScheme, VibrancyMode};

// As a general note, this function will only return a valid surface if the
// window is already visible. Otherwise, there is still no HWND yet to use.
fn get_hwnd(win: &gtk::Window) -> Option<HWND> {
    win.surface()
        .and_downcast::<Win32Surface>()
        .map(|surface| surface.handle().0 as HWND)
}

pub fn set_window_theme(win: &gtk::Window, color_scheme: ColorScheme) {
    let Some(hwnd) = get_hwnd(win) else { return };

    // Cast into the internal boolean type used to interact with the Windows API.
    let mut winapi_dark = match color_scheme {
        ColorScheme::Dark => 1 as BOOL,
        ColorScheme::Light => 0 as BOOL,
        ColorScheme::System => {
            if helpers::is_dark_mode().expect("Can't guess dark mode") {
                1 as BOOL
            } else {
                0 as BOOL
            }
        }
    };
    unsafe {
        // The function that has to be used is DwmSetWindowAttribute.
        // https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmsetwindowattribute
        //
        // The DWMWINDOWATTRIBUTE to set is DWMWA_USE_IMMERSIVE_DARK_MODE, which according
        // to the docs is the attribute number 20. However, this will only work for Windows 11.
        // There is a legacy number used in previews that is compatible with Windows 10, and
        // this is the attribute 19. So we are setting both numbers to please both OSes.
        // https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute
        DwmSetWindowAttribute(
            hwnd as HWND,
            19,
            &mut winapi_dark as *mut _ as *mut _,
            std::mem::size_of_val(&winapi_dark) as u32,
        );
        DwmSetWindowAttribute(
            hwnd as HWND,
            20,
            &mut winapi_dark as *mut _ as *mut _,
            std::mem::size_of_val(&winapi_dark) as u32,
        );
    }
}

#[allow(unused)]
pub fn update_vibrancy(win: &gtk::Window, vibrancy_mode: VibrancyMode) {
    let Some(hwnd) = get_hwnd(win) else { return };

    if helpers::windows_build_number() < 22621 {
        return;
    }
    let mica_level = match vibrancy_mode {
        VibrancyMode::MainWindow => 4,
        VibrancyMode::Transient => 2,
        VibrancyMode::None => 1,
    };
    unsafe {
        DwmSetWindowAttribute(
            hwnd as HWND,
            38,
            &mica_level as *const _ as _,
            std::mem::size_of_val(&mica_level) as u32,
        );
    }
    if vibrancy_mode == VibrancyMode::None {
        win.remove_css_class("mica");
    } else {
        win.add_css_class("mica");
    }
}
