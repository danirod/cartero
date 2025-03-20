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
use gtk::prelude::*;

#[cfg(windows)]
use crate::win::CarteroWindow;

#[cfg(windows)]
pub fn win32_set_dark_mode(win: &CarteroWindow, dark: bool) {
    use gdk4_win32::Win32Surface;

    // Get HWND from the CarteroWindow. This only works if the window is already
    // visible, otherwise Windows will not have assigned an HWND to the window yet.
    let maybe_hwnd = win
        .surface()
        .and_downcast::<Win32Surface>()
        .map(|surface| surface.handle().0);

    if let Some(hwnd) = maybe_hwnd {
        use winapi::shared::minwindef::BOOL;
        use winapi::{shared::windef::HWND, um::dwmapi::DwmSetWindowAttribute};

        // Cast into the internal boolean type used to interact with the Windows API.
        let mut winapi_dark = if dark { 1 as BOOL } else { 0 as BOOL };
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
}
