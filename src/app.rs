// Copyright 2024 the Cartero authors
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

use adw::prelude::*;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::gdk::Display;
use gtk::gio::{self, ActionEntryBuilder, Settings};
use gtk::pango::FontDescription;
use gtk::prelude::ActionMapExtManual;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

use crate::config::{APP_ID, BASE_ID, RESOURCE_PATH};
use crate::win::CarteroWindow;
use crate::windows::SettingsDialog;

#[macro_export]
macro_rules! accelerator {
    ($accel:expr) => {
        if cfg!(target_os = "macos") {
            concat!("<Meta>", $accel)
        } else {
            concat!("<Primary>", $accel)
        }
    };
}

mod imp {
    use std::cell::OnceCell;

    use adw::prelude::*;
    use adw::subclass::application::AdwApplicationImpl;
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk::gio::Settings;
    use gtk::subclass::prelude::*;
    use gtk::subclass::{application::GtkApplicationImpl, prelude::ApplicationImpl};

    use super::*;

    #[derive(Default)]
    pub struct CarteroApplication {
        pub(super) settings: OnceCell<Settings>,

        /* Used to change the text editors font descriptors. */
        pub(super) provider: OnceCell<CssProvider>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroApplication {
        const NAME: &'static str = "CarteroApplication";
        type Type = super::CarteroApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for CarteroApplication {}

    impl ApplicationImpl for CarteroApplication {
        fn activate(&self) {
            self.parent_activate();
            let (window, is_new_window) = match self.obj().active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&self.obj()), true),
            };
            glib::spawn_future_local(glib::clone!(@weak window => async move {
                if is_new_window {
                    window.open_last_session().await;
                }
                window.present();
            }));
        }

        fn startup(&self) {
            self.parent_startup();
            gtk::Window::set_default_icon_name(APP_ID);

            let obj = self.obj();
            obj.set_accels_for_action("win.new", &[accelerator!("t")]);
            obj.set_accels_for_action("win.open", &[accelerator!("o")]);
            obj.set_accels_for_action("win.save", &[accelerator!("s")]);
            obj.set_accels_for_action("win.save-as", &[accelerator!("<Shift>s")]);
            obj.set_accels_for_action("win.close", &[accelerator!("w")]);
            obj.set_accels_for_action("win.request", &[accelerator!("Return")]);
            obj.set_accels_for_action("app.preferences", &[accelerator!("comma")]);
            obj.set_accels_for_action("app.quit", &[accelerator!("q")]);
            obj.set_accels_for_action("win.show-help-overlay", &[accelerator!("question")]);
            obj.setup_app_actions();
            obj.setup_color_scheme();
            obj.setup_font();
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);

            let (window, is_new_window) = match self.obj().active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&self.obj()), true),
            };
            let thread_files: Vec<gio::File> = files.to_vec();
            glib::spawn_future_local(async move {
                if is_new_window {
                    window.open_last_session().await;
                }
                for file in thread_files {
                    window.add_endpoint(Some(&file)).await;
                }
                window.present();
            });
        }
    }

    impl GtkApplicationImpl for CarteroApplication {}

    impl AdwApplicationImpl for CarteroApplication {}
}

glib::wrapper! {
    pub struct CarteroApplication(ObjectSubclass<imp::CarteroApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;

}

impl Default for CarteroApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl CarteroApplication {
    pub fn get() -> Self {
        gio::Application::default()
            .and_downcast::<CarteroApplication>()
            .unwrap()
    }

    pub fn new() -> Self {
        Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .property("resource-base-path", RESOURCE_PATH)
            .build()
    }

    pub fn settings(&self) -> &Settings {
        self.imp().settings.get_or_init(|| Settings::new(BASE_ID))
    }

    pub fn css_provider(&self) -> &CssProvider {
        self.imp().provider.get_or_init(|| CssProvider::new())
    }

    fn setup_color_scheme(&self) {
        let settings = self.settings();
        settings
            .bind("application-theme", &self.style_manager(), "color-scheme")
            .mapping(|val, _| {
                let scheme = match val.get::<String>().unwrap().as_str() {
                    "light" => adw::ColorScheme::ForceLight,
                    "dark" => adw::ColorScheme::ForceDark,
                    _ => adw::ColorScheme::Default,
                };
                Some(scheme.into())
            })
            .build();
    }

    fn setup_font(&self) {
        let css_provider = self.css_provider();
        let display = Display::default().expect("No display available");
        gtk::style_context_add_provider_for_display(
            &display,
            css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let settings = self.settings();
        settings.connect_changed(
            None,
            glib::clone!(@weak self as app => move |_, _| {
                app.update_font();
            }),
        );
        self.style_manager()
            .connect_dark_notify(glib::clone!(@weak self as app => move |_| {
                app.update_font();
            }));
    }

    fn update_font(&self) {
        let current_font_descriptor = self.settings().get::<String>("custom-font");
        let system_font = self.settings().get::<bool>("use-system-font");

        let provider = self.css_provider();

        let css: String = if system_font {
            "".into()
        } else {
            let descriptor = FontDescription::from_string(&current_font_descriptor);
            let font_family: String = descriptor.family().unwrap_or_default().into();
            let font_size = descriptor.size() / gtk::pango::SCALE;
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
                textview.use-cartero-font {{
                    font-family: "{font_family}";
                    font-size: {font_size}pt;
                    font-style: {font_style};
                    font-weight: {font_weight};
                }}"#
            )
        };
        provider.load_from_string(&css);
    }

    fn setup_app_actions(&self) {
        let settings = ActionEntryBuilder::new("preferences")
            .activate(glib::clone!(@weak self as app => move |_, _, _| {
                if let Some(window) = app.active_window() {
                    SettingsDialog::present_for_window(&window);
                }
            }))
            .build();
        let quit = ActionEntryBuilder::new("quit")
            .activate(glib::clone!(@weak self as app => move |_, _, _| {
                for window in app.windows() {
                    window.close();
                }

                if app.windows().is_empty() {
                    app.quit();
                }
            }))
            .build();

        self.add_action_entries([settings, quit]);
    }
}
