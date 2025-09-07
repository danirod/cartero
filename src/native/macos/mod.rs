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
use glib::object::{CastNone, ObjectExt};
use gtk::prelude::{NativeExt, WidgetExt};
use objc2::{class, msg_send, runtime::AnyObject};
use objc2_app_kit::{NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSView, NSVisualEffectMaterial, NSVisualEffectView, NSWindow};
use objc2_core_graphics::CGColor;
use objc2_foundation::{NSInteger, NSRect};

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

pub(super) fn set_vibrancy(win: &gtk::Window, vibrancy: super::VibrancyMode) {
    if let Some(ns_window) = ns_window(win) {
        unsafe {
            if let Some(old_backdrop) = win.steal_data::<*mut NSVisualEffectView>("backdrop") {
                let _: () = msg_send![old_backdrop, removeFromSuperview];
                win.remove_css_class("vibrant");
            }
            let material = match vibrancy {
                super::VibrancyMode::None => None,
                super::VibrancyMode::Standard => Some(NSVisualEffectMaterial::WindowBackground.0 as NSInteger),
                super::VibrancyMode::Sidebar => Some(NSVisualEffectMaterial::Sidebar.0 as NSInteger),
            };

            let view: *mut NSView = msg_send![ns_window, contentView];
            let bounds: NSRect = msg_send![view, bounds];
            if let Some(material) = material {
                let alloc_view: *mut AnyObject = msg_send![class!(NSVisualEffectView), alloc];
                let backdrop: *mut NSVisualEffectView = msg_send![alloc_view, initWithFrame: bounds];

                let _: () = msg_send![backdrop, setMaterial: material];
                let _: () = msg_send![backdrop, setBlendingMode: 0 as NSInteger];
                let _: () = msg_send![backdrop, setState: 1 as NSInteger];
                let _: () = msg_send![backdrop, setAutoresizingMask: 18 as NSInteger];
                let _: () = msg_send![backdrop, setWantsLayer: true];

                let backdrop_layer: *mut AnyObject = msg_send![backdrop, layer];
                let _: () = msg_send![backdrop_layer, setOpaque: false];
                let _: () = msg_send![backdrop_layer, setZPosition: -9999 as f64];
                let clear_color: *mut AnyObject = msg_send![class!(NSColor), clearColor];
                let cgcolor: *mut CGColor = msg_send![clear_color, CGColor];
                let _: () = msg_send![backdrop_layer, setBackgroundColor: cgcolor];

                let _: () = msg_send![view, addSubview: backdrop];
                win.set_data("backdrop", backdrop);

                win.add_css_class("vibrant");
            }
        }
    }
}