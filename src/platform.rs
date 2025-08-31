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

#[cfg(all(windows, not(feature = "csd")))]
#[derive(Copy, Clone)]
pub enum MicaLevel {
    MainWindow = 2,
    Tabbed = 4,
}

#[cfg(all(windows, not(feature = "csd")))]
fn win32_get_build_number() -> u32 {
    // You use the RtlGetVersion. There was a GetVersion, but it is currently deprecated.
    // I know it will never stop working, but just in case let's not use it.
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-rtlgetversion
    use std::sync::OnceLock;
    use windows_sys::Wdk::System::SystemServices::RtlGetVersion;
    use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;

    static BUILD_NUMBER: OnceLock<u32> = OnceLock::new();
    *BUILD_NUMBER.get_or_init(|| {
        // The C version of RtlGetVersion literally receives a pointer so that it can dump
        // OS information. Standard behaviour for C, but since we are crossing the FFI
        // frontier here, it has to be done exactly like that too. Spooky.
        let version_info = OSVERSIONINFOW {
            ..Default::default()
        };
        unsafe {
            if RtlGetVersion(&version_info as *const _ as _) == 0 {
                version_info.dwBuildNumber
            } else {
                0
            }
        }
    })
}

#[cfg(all(windows, not(feature = "csd")))]
fn win32_supports_mica() -> bool {
    // Mica is only supported on Windows 11 Build 22621 or greater:
    // https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute
    win32_get_build_number() >= 22621
}

#[cfg(all(windows, not(feature = "csd")))]
pub fn win32_set_mica(win: &gtk::Window, mica: MicaLevel) {
    use gdk4_win32::Win32Surface;
    use gtk::prelude::*;

    // Get HWND from the window. This only works if the window is already
    // visible, otherwise Windows will not have assigned an HWND to the window yet.
    let maybe_hwnd = win
        .surface()
        .and_downcast::<Win32Surface>()
        .map(|surface| surface.handle().0);

    if let Some(hwnd) = maybe_hwnd {
        use winapi::{shared::windef::HWND, um::dwmapi::DwmSetWindowAttribute};

        let winapi_backdrop = mica as i32;
        unsafe {
            DwmSetWindowAttribute(
                hwnd as HWND,
                38,
                &winapi_backdrop as *const _ as _,
                std::mem::size_of_val(&winapi_backdrop) as u32,
            );
        }
    }
}

#[cfg(all(windows, not(feature = "csd")))]
pub fn win32_set_dark_mode(win: &gtk::Window, dark: bool) {
    use gdk4_win32::Win32Surface;
    use gtk::prelude::*;

    // Get HWND from the window. This only works if the window is already
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

#[cfg(all(windows, not(feature = "csd")))]
pub fn win32_init_window(window: &gtk::Window, mica: MicaLevel) {
    use gtk::prelude::*;

    let style_manager = adw::StyleManager::default();
    win32_set_dark_mode(window, style_manager.is_dark());
    if win32_supports_mica() {
        win32_set_mica(window, mica.clone());
        if !style_manager.is_high_contrast() {
            window.add_css_class("mica");
        }
    }

    style_manager.connect_color_scheme_notify(glib::clone!(
        #[weak]
        window,
        move |scheme: &adw::StyleManager| {
            let dark = scheme.is_dark();
            win32_set_dark_mode(&window, dark);
        }
    ));
    style_manager.connect_dark_notify(glib::clone!(
        #[weak]
        window,
        move |scheme: &adw::StyleManager| {
            let dark = scheme.is_dark();
            win32_set_dark_mode(&window, dark);
        }
    ));
    style_manager.connect_high_contrast_notify(glib::clone!(
        #[weak]
        window,
        move |scheme: &adw::StyleManager| {
            let dark = scheme.is_dark();
            win32_set_dark_mode(&window, dark);
            if scheme.is_high_contrast() {
                window.remove_css_class("mica");
            } else {
                window.add_css_class("mica");
            }
        }
    ));

    let mica_clone = mica.clone();
    window.connect_realize(glib::clone!(
        #[weak]
        style_manager,
        move |window| {
            let dark = style_manager.is_dark();
            win32_set_mica(&window, mica_clone);
            win32_set_dark_mode(&window, dark);
        }
    ));
}
