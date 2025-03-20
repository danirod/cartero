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

use glib::{object::ObjectExt, Object};
use gtk::{glib, pango::FontDescription, prelude::SettingsExtManual};

use crate::app::CarteroApplication;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use glib::object::{Cast, ObjectExt};
    use glib::subclass::Signal;
    use glib::value::ToValue;
    use glib::Properties;
    use gtk::gdk::ModifierType;
    use gtk::gio::SettingsBindFlags;
    #[allow(deprecated)]
    use gtk::prelude::StyleContextExt;
    use gtk::prelude::{EventControllerExt, WidgetExt};
    use gtk::prelude::{SettingsExt, SettingsExtManual, TextViewExt};
    use gtk::subclass::prelude::*;
    use gtk::{gdk, EventControllerScroll, EventControllerScrollFlags, PropagationPhase};
    use gtk::{glib, WrapMode};
    use sourceview5::prelude::BufferExt;
    use sourceview5::subclass::view::ViewImpl;
    use sourceview5::StyleSchemeManager;

    use crate::app::CarteroApplication;
    use crate::widgets::code_view::{get_monospace_font_descriptor, render_css_rules};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::CodeView)]
    pub struct CodeView {
        #[property(get, set, name = "zoom-level")]
        zoom_level: RefCell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CodeView {
        const NAME: &'static str = "CarteroCodeView";
        type Type = super::CodeView;
        type ParentType = sourceview5::View;

        fn class_init(klass: &mut Self::Class) {
            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                "codeview.search",
            );
            klass.add_binding_action(
                gdk::Key::plus,
                gdk::ModifierType::CONTROL_MASK,
                "widget.zoom-in",
            );
            klass.add_binding_action(
                gdk::Key::minus,
                gdk::ModifierType::CONTROL_MASK,
                "widget.zoom-out",
            );
            klass.add_binding_action(
                gdk::Key::_0,
                gdk::ModifierType::CONTROL_MASK,
                "widget.zoom-reset",
            );

            klass.install_action("codeview.search", None, |widget, _, _| {
                widget.start_search();
            });
            klass.install_action("widget.zoom-in", None, |widget, _, _| {
                let old_zoom = widget.zoom_level();
                if old_zoom < 18 {
                    widget.set_zoom_level(old_zoom + 1);
                }
            });
            klass.install_action("widget.zoom-out", None, |widget, _, _| {
                let old_zoom = widget.zoom_level();
                if old_zoom > -6 {
                    widget.set_zoom_level(old_zoom - 1);
                }
            });
            klass.install_action("widget.zoom-reset", None, |widget, _, _| {
                widget.set_zoom_level(0);
            });
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CodeView {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("search-requested").build()])
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
            self.init_source_view_css();
            self.init_source_view_style();
            self.init_scroll_controller();
        }
    }

    impl WidgetImpl for CodeView {}

    impl TextViewImpl for CodeView {}

    impl ViewImpl for CodeView {}

    impl CodeView {
        fn init_scroll_controller(&self) {
            let obj = self.obj();
            let controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
            controller.set_propagation_phase(PropagationPhase::Capture);
            controller.connect_scroll(glib::clone!(
                #[weak]
                obj,
                #[upgrade_or_panic]
                move |controller: &EventControllerScroll, _dx: f64, dy: f64| {
                    /* Only interested in CTRL + scroll events. */
                    let state = controller.current_event_state();
                    if state == ModifierType::CONTROL_MASK {
                        if dy > 0.0 {
                            obj.activate_action("widget.zoom-out", None).unwrap();
                        } else {
                            obj.activate_action("widget.zoom-in", None).unwrap();
                        }
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                }
            ));
            obj.add_controller(controller);
        }

        fn init_settings(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();
            let obj = self.obj();

            settings
                .bind("body-wrap", &*obj, "wrap-mode")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let enabled = variant.get::<bool>().expect("The variant is not a boolean");
                    let mode = match enabled {
                        true => WrapMode::WordChar,
                        false => WrapMode::None,
                    };
                    Some(mode.to_value())
                })
                .build();
            settings
                .bind("show-line-numbers", &*obj, "show-line-numbers")
                .flags(SettingsBindFlags::GET)
                .build();
            settings
                .bind("auto-indent", &*obj, "auto-indent")
                .flags(SettingsBindFlags::GET)
                .build();
            settings
                .bind("indent-style", &*obj, "insert-spaces-instead-of-tabs")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let mode = variant
                        .get::<String>()
                        .expect("The variant is not a string");
                    let use_spaces = mode == "spaces";
                    Some(use_spaces.to_value())
                })
                .build();
            settings
                .bind("tab-width", &*obj, "tab-width")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let width = variant.get::<String>().unwrap_or("4".into());
                    let value = width.parse::<i32>().unwrap_or(4);
                    Some(value.to_value())
                })
                .build();
            settings
                .bind("tab-width", &*obj, "indent-width")
                .flags(SettingsBindFlags::GET)
                .mapping(|variant, _| {
                    let width = variant.get::<String>().unwrap_or("4".into());
                    let value = width.parse::<i32>().unwrap_or(4);
                    Some(value.to_value())
                })
                .build();
        }

        /// Renders the whole CSS string that will be attached to the widget instance.
        fn generate_css(&self) -> String {
            let obj = self.obj();
            let zoom_level = obj.zoom_level();
            let font = get_monospace_font_descriptor();
            render_css_rules(&font, zoom_level)
        }

        /// Configures the initial CSS style for this widget, and also setups the callbacks
        /// to reconsider the CSS style whenever the configuration changes.
        fn init_source_view_css(&self) {
            let obj = self.obj();

            /* First, register the CSS provider attached to this view. */
            let provider = gtk::CssProvider::new();
            #[allow(deprecated)] // eat shit
            obj.style_context()
                .add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

            /* Set the initial CSS style. */
            let css = self.generate_css();
            provider.load_from_string(&css);

            /* Whenever the settings change, we also need to update the font. */
            let app = CarteroApplication::get();
            let settings = app.settings();
            settings.connect_changed(
                None,
                glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    #[weak]
                    provider,
                    move |_, key: &str| {
                        if key == "use-system-font" || key == "custom-font" {
                            let css = imp.generate_css();
                            provider.load_from_string(&css);
                        }
                    }
                ),
            );

            obj.connect_zoom_level_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    let css = imp.generate_css();
                    provider.load_from_string(&css);
                }
            ));
        }

        fn update_source_view_style(&self) {
            let obj = self.obj();
            let dark_mode = adw::StyleManager::default().is_dark();
            let color_theme = if dark_mode { "Adwaita-dark" } else { "Adwaita" };
            let theme = StyleSchemeManager::default().scheme(color_theme);
            let buffer = obj.buffer().downcast::<sourceview5::Buffer>().unwrap();
            match theme {
                Some(theme) => {
                    buffer.set_style_scheme(Some(&theme));
                    buffer.set_highlight_syntax(true);
                }
                None => {
                    buffer.set_highlight_syntax(false);
                }
            }
        }

        fn init_source_view_style(&self) {
            self.update_source_view_style();
            adw::StyleManager::default().connect_dark_notify(glib::clone!(
                #[weak(rename_to = panel)]
                self,
                move |_| {
                    panel.update_source_view_style();
                }
            ));
            let obj = self.obj();
            obj.connect_buffer_notify(glib::clone!(
                #[weak(rename_to = panel)]
                self,
                move |_| {
                    panel.update_source_view_style();
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct CodeView(ObjectSubclass<imp::CodeView>)
        @extends gtk::Widget, gtk::TextView, sourceview5::View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

impl CodeView {
    pub fn start_search(&self) {
        self.emit_by_name::<()>("search-requested", &[]);
    }
}

impl Default for CodeView {
    fn default() -> Self {
        Object::builder().build()
    }
}

/// Returns the pango font descriptor used to render monospaced text views.
/// If the user has customized the font in the settings, that's the descriptor
/// that will be returned, otherwise just returns the default system one.
fn get_monospace_font_descriptor() -> FontDescription {
    let app = CarteroApplication::get();
    let settings = app.settings();
    let use_system_font = settings.get::<bool>("use-system-font");

    let mono_font = if use_system_font {
        if cfg!(target_os = "macos") {
            "Menlo 14".to_string()
        } else if cfg!(target_os = "windows") {
            "Consolas 11".to_string()
        } else {
            let settings = gtk::gio::Settings::new("org.gnome.desktop.interface");
            settings.get::<String>("monospace-font-name")
        }
    } else {
        settings.get::<String>("custom-font")
    };
    FontDescription::from_string(&mono_font)
}

/// Given the zoom level, returns the scaling percentage.
fn map_zoom_level(zoom: i32) -> f32 {
    let perc = 1.0 + (zoom as f32 / 8.0);
    if perc < 0.25 {
        0.25
    } else {
        perc
    }
}

/// Returns the CSS document that should be provided to the StyleContext of a CodeView
/// in order to tweak how the view should appear, based on the font settings and the
/// zoom level in use.
fn render_css_rules(descriptor: &FontDescription, zoom: i32) -> String {
    let font_family: String = descriptor.family().unwrap_or_default().into();
    let font_size = descriptor.size() as f32 * map_zoom_level(zoom);
    let real_font_size = font_size as i32 / gtk::pango::SCALE;
    let font_style = {
        match descriptor.style() {
            gtk::pango::Style::Italic => "italic",
            gtk::pango::Style::Oblique => "oblique",
            _ => "normal",
        }
    };
    let font_weight = {
        match descriptor.weight() {
            gtk::pango::Weight::Bold => 700,
            gtk::pango::Weight::Book => 380,
            gtk::pango::Weight::Heavy => 900,
            gtk::pango::Weight::Light => 300,
            gtk::pango::Weight::Medium => 500,
            gtk::pango::Weight::Normal => 400,
            gtk::pango::Weight::Semibold => 600,
            gtk::pango::Weight::Semilight => 350,
            gtk::pango::Weight::Thin => 100,
            gtk::pango::Weight::Ultrabold => 800,
            gtk::pango::Weight::Ultraheavy => 1000,
            gtk::pango::Weight::Ultralight => 200,
            gtk::pango::Weight::__Unknown(i) => i,
            _ => 400,
        }
    };
    format!(
        r#"
          textview {{
              font-family: "{font_family}";
              font-size: {real_font_size}pt;
              font-style: {font_style};
              font-weight: {font_weight};
          }}
        "#
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_map_zoom_level() {
        let cases = vec![
            (-6, 0.25), // Cannot go negative or zero
            (-4, 0.5),
            (-2, 0.75),
            (0, 1.0),
            (1, 1.125),
            (2, 1.25),
            (4, 1.5),
            (8, 2.0),
            (10, 2.25),
        ];
        for (input, output) in cases {
            assert_eq!(output, super::map_zoom_level(input));
        }
    }
}
