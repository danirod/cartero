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
use gtk::prelude::{GtkWindowExt, NativeExt, WidgetExt};
use objc2::{class, msg_send, runtime::AnyObject};
use objc2_app_kit::{NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectView, NSWindow};
use objc2_core_foundation::{CGRect, CGSize};
use objc2_core_graphics::CGColor;
use objc2_foundation::{NSInteger, NSPoint};

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

unsafe fn initialize_backdrop(material: NSVisualEffectMaterial, blending: NSVisualEffectBlendingMode) -> *mut NSVisualEffectView {
    let backdrop: *mut NSVisualEffectView = msg_send![class!(NSVisualEffectView), new];

    let _: () = msg_send![backdrop, setMaterial: material.0 as NSInteger];
    let _: () = msg_send![backdrop, setBlendingMode: blending.0 as NSInteger];
    let _: () = msg_send![backdrop, setState: 1 as NSInteger];
    let _: () = msg_send![backdrop, setAutoresizingMask: 18 as NSInteger];
    let _: () = msg_send![backdrop, setWantsLayer: true];

    let backdrop_layer: *mut AnyObject = msg_send![backdrop, layer];
    let _: () = msg_send![backdrop_layer, setOpaque: false];
    let _: () = msg_send![backdrop_layer, setZPosition: -9999 as f64];
    let clear_color: *mut AnyObject = msg_send![class!(NSColor), clearColor];
    let cgcolor: *mut CGColor = msg_send![clear_color, CGColor];
    let _: () = msg_send![backdrop_layer, setBackgroundColor: cgcolor];

    backdrop
}

pub(super) fn update_vibrancy(win: &gtk::Window, sidebar: Option<i32>, headerbar: Option<i32>) {
    let window_height = win.default_height() as f64;
    let corner_x = sidebar.unwrap_or_default() as f64;
    let area_width = (win.default_width() - sidebar.unwrap_or_default()) as f64;

    let Some(ns_window) = ns_window(win) else {
        return;
    };

    unsafe {
        // Disable any kind of animation.
        let _: () = msg_send![class!(CATransaction), begin];
        let _: () = msg_send![class!(CATransaction), setDisableActions: true];

        let view: *mut NSView = msg_send![ns_window, contentView];

        // Has a Sidebar?
        match sidebar {
            Some(sidebar_width) => {
                // Add if not present.
                let sidebar_view = win.data::<*mut NSVisualEffectView>("sidebar-backdrop")
                    .map(|pointer| *pointer.as_ptr())
                    .unwrap_or_else(|| {
                        let backdrop = initialize_backdrop(NSVisualEffectMaterial::Sidebar, NSVisualEffectBlendingMode::BehindWindow);
                        win.set_data("sidebar-backdrop", backdrop);
                        let _: () = msg_send![view, addSubview: backdrop];
                        backdrop
                    });
                let sidebar_view_frame = {
                    let pointer = NSPoint::ZERO;
                    let sizes = CGSize::new(sidebar_width as f64, window_height);
                    CGRect::new(pointer, sizes)
                };
                let _: () = msg_send![sidebar_view, setFrame: sidebar_view_frame];
            },
            None => {
                // Remove if present.
                if let Some(sidebar) = win.steal_data::<*mut NSVisualEffectView>("sidebar-backdrop") {
                    let _: () = msg_send![sidebar, removeFromSuperview];
                }
            },
        }

        // Main backdrop.
        let main_backdrop = win.data::<*mut NSVisualEffectView>("main-backdrop")
            .map(|pointer| *pointer.as_ptr())
            .unwrap_or_else(|| {
                let backdrop = initialize_backdrop(NSVisualEffectMaterial::WindowBackground, NSVisualEffectBlendingMode::BehindWindow);
                win.set_data("main-backdrop", backdrop);
                let _: () = msg_send![view, addSubview: backdrop];
                backdrop
            });
        let main_backdrop_frame = {
            let pointer = NSPoint::new(corner_x, 0 as f64);
            let sizes = CGSize::new(area_width, window_height);
            CGRect::new(pointer, sizes)
        };
        let _: () = msg_send![main_backdrop, setFrame: main_backdrop_frame];

        // Has a HeaderView?
        match headerbar {
            Some(headerbar_height) => {
                // Add if not present.
                let header_view = win.data::<*mut NSVisualEffectView>("header-backdrop")
                    .map(|pointer| *pointer.as_ptr())
                    .unwrap_or_else(|| {
                        let backdrop = initialize_backdrop(NSVisualEffectMaterial::Titlebar, NSVisualEffectBlendingMode::WithinWindow);
                        win.set_data("header-backdrop", backdrop);
                        let _: () = msg_send![view, addSubview: backdrop];
                        backdrop
                    });
                let header_view_frame = {
                    let pointer = NSPoint::ZERO;
                    let sizes = CGSize::new(area_width, headerbar_height as f64);
                    CGRect::new(pointer, sizes)
                };
                let _: () = msg_send![header_view, setFrame: header_view_frame];
            },
            None => {
                // Remove if present.
                if let Some(headerbar) = win.steal_data::<*mut NSVisualEffectView>("header-backdrop") {
                    let _: () = msg_send![headerbar, removeFromSuperview];
                }
            },
        }

        let _: () = msg_send![class!(CATransaction), commit];
    }

    if !win.has_css_class("vibrant") {
        win.add_css_class("vibrant");
    }
}
