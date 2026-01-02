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

use std::sync::OnceLock;
use windows_sys::Wdk::System::SystemServices::RtlGetVersion;
use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

#[allow(unused)]
pub(super) fn windows_build_number() -> u32 {
    // You use the RtlGetVersion. There was a GetVersion, but it is currently deprecated.
    // I know it will never stop working, but just in case let's not use it.
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-rtlgetversion
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

pub(super) fn is_dark_mode() -> std::io::Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let light_theme =
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")?;
    let is_light = light_theme.get_value::<u32, _>("AppsUseLightTheme")?;
    Ok(is_light == 0)
}
