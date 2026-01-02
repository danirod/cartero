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

use glib::object::ObjectExt;
use gtk::gdk::RGBA;
use sourceview5::StyleScheme;

pub(crate) fn generate_window_style(scheme: &StyleScheme) -> String {
    // TODO: Provide alternative versions of window_style and even disable.
    generate_window_css(scheme)
}

fn generate_window_css(scheme: &StyleScheme) -> String {
    let mut css = CssTheme::new();
    let dark_mode = adw::StyleManager::default().is_dark();
    static FORCE_WHITE: RGBA = RGBA::new(1.0, 1.0, 1.0, 1.0);
    static FORCE_BLACK: RGBA = RGBA::new(0.0, 0.0, 0.0, 1.0);
    let Some(text_bg) = locate_scheme_color(scheme, "text", "background") else {
        return css.to_string();
    };
    let Some(text_fg) = locate_scheme_color(scheme, "text", "foreground") else {
        return css.to_string();
    };

    if dark_mode {
        css.variable("window_bg_color", &text_bg);
    } else {
        css.clamp("window_bg_color", &text_bg, &FORCE_WHITE, 0.03);
    }
    css.variable("window_fg_color", &text_fg);
    css.variable("headerbar_bg_color", &text_bg);
    css.variable("headerbar_fg_color", &text_fg);
    css.variable("sidebar_fg_color", &text_fg);
    if dark_mode {
        css.clamp("sidebar_bg_color", &text_bg, &FORCE_WHITE, 0.05);
        css.clamp("view_bg_color", &text_bg, &FORCE_WHITE, 0.05);
        css.clamp("dialog_bg_color", &text_bg, &FORCE_WHITE, 0.05);
        css.clamp("popover_bg_color", &text_bg, &FORCE_WHITE, 0.05);
        css.alias("card_bg_color", "alpha(white, .08)");
    } else {
        css.clamp("sidebar_bg_color", &text_bg, &FORCE_BLACK, 0.05);
        css.clamp("view_bg_color", &text_bg, &FORCE_BLACK, 0.03);
        css.clamp("dialog_bg_color", &text_bg, &FORCE_BLACK, 0.1);
        css.clamp("popover_bg_color", &text_bg, &FORCE_WHITE, 0.15);
        css.alias("card_bg_color", "alpha(white, .5)");
    }

    match (
        locate_meta_color(scheme, "accent_bg_color").or(locate_scheme_color(
            scheme,
            "selection",
            "background",
        )),
        locate_meta_color(scheme, "accent_fg_color").or(locate_scheme_color(
            scheme,
            "selection",
            "foreground",
        )),
    ) {
        (Some(accent_bg), Some(accent_fg)) => {
            if !accent_bg.is_clear() {
                css.variable("accent_bg_color", &accent_bg);
                if accent_fg.is_clear() {
                    css.variable("accent_fg_color", &text_fg);
                } else {
                    css.variable("accent_fg_color", &accent_fg);
                }
            }
        }
        _ => {}
    }

    css.alias("card_fg_color", "@window_fg_color");
    css.alias("headerbar_border_color", "@window_fg_color");
    css.alias("popover_fg_color", "@window_fg_color");
    css.alias("dialog_fg_color", "@window_fg_color");
    css.alias("dark_fill_bg_color", "@headerbar_bg_color");
    css.alias("sidebar_backdrop_color", "@window_bg_color");
    css.alias("sidebar_fg_color", "@window_fg_color");
    css.alias("view_fg_color", "@window_fg_color");

    css.to_string()
}

fn locate_meta_color(scheme: &StyleScheme, name: &str) -> Option<RGBA> {
    scheme
        .metadata(name)
        .and_then(|value| RGBA::parse(value).ok())
}

fn locate_scheme_color(scheme: &StyleScheme, style: &str, property: &str) -> Option<RGBA> {
    scheme
        .style(style)
        .and_then(|style| style.property::<Option<glib::GString>>(property))
        .and_then(|color| RGBA::parse(color).ok())
}

struct CssTheme(String);

impl CssTheme {
    fn new() -> Self {
        Self(String::new())
    }

    fn alias(&mut self, variable: &str, from: &str) {
        self.0
            .push_str(&format!("@define-color {} {};\n", variable, from));
    }

    fn variable(&mut self, variable: &str, color: &RGBA) {
        let hex = color.to_string();
        self.0
            .push_str(&format!("@define-color {} {};\n", variable, hex));
    }

    fn clamp(&mut self, variable: &str, a: &RGBA, b: &RGBA, clamp: f32) {
        let hex_a = a.to_string();
        let hex_b = b.to_string();
        let clamp_level = format!("{:.2}", clamp);
        self.0.push_str(&format!(
            "@define-color {} mix({}, {}, {});\n",
            variable, hex_a, hex_b, clamp_level
        ));
    }

    fn to_string(&self) -> String {
        self.0.clone()
    }
}
