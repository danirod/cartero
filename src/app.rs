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

use adw::prelude::*;
use gettextrs::gettext;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::gio::{self, ActionEntryBuilder};
use gtk::prelude::ActionMapExtManual;
#[allow(deprecated)]
use gtk::StyleContext;
use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
use sourceview5::StyleSchemeManager;

use crate::config::{APP_ID, RESOURCE_PATH};
use crate::settings::Settings;
use crate::win::CarteroWindow;
use crate::windows::common::get_window_shell;

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
    use gtk::subclass::prelude::*;
    use gtk::subclass::{application::GtkApplicationImpl, prelude::ApplicationImpl};
    use gtk::CssProvider;

    use super::*;

    #[derive(Default)]
    pub struct CarteroApplication {
        pub(super) settings: Settings,
        pub(super) app_theme: OnceCell<CssProvider>,
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
            let obj = self.obj();

            let (window, is_new_window) = match obj.active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&obj), true),
            };

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if is_new_window {
                        let last_session = obj.last_session_tabs();
                        let open_result = window.open_endpoints(&last_session).await;
                        window.present();
                        window.report_open_endpoints_errors(&open_result).await;
                    } else {
                        window.present();
                    }
                }
            ));
        }

        fn startup(&self) {
            self.parent_startup();
            gtk::Window::set_default_icon_name(APP_ID);

            if cfg!(target_os = "windows") {
                if let Some(settings) = gtk::Settings::default() {
                    settings.set_gtk_font_name(Some("Segoe UI 10"));
                }
            } else if cfg!(target_os = "macos") {
                if let Some(settings) = gtk::Settings::default() {
                    settings.set_gtk_font_name(Some(".AppleSystemUIFont 14.5"));
                }
            }

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
        }

        fn open(&self, files: &[gio::File], hint: &str) {
            self.parent_open(files, hint);
            let obj = self.obj();
            let (window, is_new_window) = match self.obj().active_window() {
                Some(window) => (window.downcast::<CarteroWindow>().unwrap(), false),
                None => (CarteroWindow::new(&self.obj()), true),
            };

            /* If it's a new window, also open the files from the previous session. */
            let files_to_open: Vec<gio::File> = if is_new_window {
                let mut previous_files = obj.last_session_tabs();
                previous_files.extend(files.to_vec());
                previous_files
            } else {
                files.to_vec()
            };

            glib::spawn_future_local(async move {
                let open_result = window.open_endpoints(&files_to_open).await;
                window.present();
                window.report_open_endpoints_errors(&open_result).await;
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
    pub fn windows_by_type<T>(&self) -> Vec<gtk::Window>
    where
        T: IsA<gtk::Widget>,
    {
        self.windows()
            .into_iter()
            .filter(|win| match win.downcast_ref::<adw::ApplicationWindow>() {
                Some(adw_win) => adw_win.content().and_downcast_ref::<T>().is_some(),
                None => win.child().and_downcast_ref::<T>().is_some(),
            })
            .collect::<Vec<gtk::Window>>()
    }

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

    fn setup_color_scheme(&self) {
        self.imp()
            .settings
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

        #[allow(deprecated)]
        StyleContext::add_provider_for_display(
            &gtk::gdk::Display::default().expect("No display"),
            self.imp().app_theme.get_or_init(|| gtk::CssProvider::new()),
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.update_app_css();
        self.imp().settings.connect_changed(
            Some("color-scheme-dark"),
            glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _| {
                    app.update_app_css();
                }
            ),
        );
        self.imp().settings.connect_changed(
            Some("color-scheme-light"),
            glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _| {
                    app.update_app_css();
                }
            ),
        );
        self.style_manager().connect_dark_notify(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |_| {
                app.update_app_css();
            }
        ));
    }

    fn update_app_css(&self) {
        let mut css = String::new();

        let style = if self.style_manager().is_dark() {
            self.imp().settings.get::<String>("color-scheme-dark")
        } else {
            self.imp().settings.get::<String>("color-scheme-light")
        };
        if let Some(scheme) = StyleSchemeManager::default().scheme(style.as_ref()) {
            if let Some(text) = scheme.style("text") {
                if let Some(fg) = text.foreground() {
                    css.push_str(&format!("@define-color window_fg_color {};", fg.as_str()));
                    css.push_str(&format!(
                        "@define-color headerbar_fg_color {};",
                        fg.as_str()
                    ));
                    css.push_str(&format!("@define-color sidebar_fg_color {};", fg.as_str()));
                    css.push_str(&format!("@define-color view_fg_color {};", fg.as_str()));
                }
                if let Some(bg) = text.background() {
                    css.push_str(&format!("@define-color window_bg_color {};", bg.as_str()));
                    css.push_str(&format!("@define-color view_bg_color {};", bg.as_str()));
                }
            }

            if let Some(line_number) = scheme.style("current-line") {
                if let Some(bg) = line_number.background() {
                    css.push_str(&format!(
                        "@define-color headerbar_bg_color {};",
                        bg.as_str()
                    ));
                    css.push_str(&format!("@define-color sidebar_bg_color {};", bg.as_str()));
                }
            }
        }

        self.imp()
            .app_theme
            .get_or_init(|| gtk::CssProvider::new())
            .load_from_string(css.as_str());
    }

    fn setup_app_actions(&self) {
        let settings = ActionEntryBuilder::new("preferences")
            .activate(glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _, _| {
                    let current_window = app.active_window();
                    let window = app
                        .windows_by_type::<crate::windows::settings::Shell>()
                        .first()
                        .map(|win| win.clone())
                        .unwrap_or_else(|| {
                            let settings_shell = crate::windows::settings::Shell::new();
                            let window = app.new_window(&settings_shell);
                            window.set_modal(true);
                            window.set_default_size(700, 500);
                            window.set_title(Some(&gettext("Settings")));
                            window.set_resizable(false);
                            {
                                // Prepare native L&F
                                let base_window = window.clone().upcast::<gtk::Window>();
                                settings_shell.init_native_window(&base_window);
                            }
                            window.upcast()
                        });
                    window.set_transient_for(current_window.as_ref());
                    window.present();
                }
            ))
            .build();
        let settings_page = ActionEntryBuilder::new("preferences-page")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _, page| {
                    let page = page
                        .map(|v| v.get::<String>().expect("Missing parameter "))
                        .unwrap_or("application".to_string());
                    let window = app
                        .windows_by_type::<crate::windows::settings::Shell>()
                        .first()
                        .map(|win| win.clone())
                        .unwrap_or_else(|| {
                            let settings_shell = crate::windows::settings::Shell::new();
                            let window = app.new_window(&settings_shell);
                            window.set_modal(true);
                            window.set_default_size(700, 500);
                            window.set_title(Some(&gettext("Settings")));
                            window.set_resizable(false);
                            window.upcast()
                        });
                    let shell = get_window_shell(&window)
                        .and_downcast::<crate::windows::settings::Shell>()
                        .expect("No settings shell?");
                    shell.set_page(&page);
                    window.present();
                }
            ))
            .build();
        let about = ActionEntryBuilder::new("about")
            .activate(|app: &Self, _, _| {
                if let Some(window) = app.active_window() {
                    let _ = window.activate_action("win.about", None);
                }
            })
            .build();
        let quit = ActionEntryBuilder::new("quit")
            .activate(glib::clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _, _| {
                    for window in app.windows() {
                        window.close();
                    }

                    if app.windows().is_empty() {
                        app.quit();
                    }
                }
            ))
            .build();

        #[cfg(feature = "app_updater")]
        {
            let action_check_updates = ActionEntryBuilder::new("check-updates")
                .activate(|app: &CarteroApplication, _, _| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        app,
                        async move {
                            let action = app
                                .lookup_action("check-updates")
                                .expect("Action not registered?");
                            action.set_property("enabled", false);
                            app.action_check_updates().await;
                            action.set_property("enabled", true);
                        }
                    ));
                })
                .build();
            self.add_action_entries([action_check_updates]);
        }

        self.add_action_entries([settings, settings_page, about, quit]);

        if cfg!(target_os = "macos") {
            let links = vec![
                ("menubar.user-manual", "https://cartero.danirod.es/docs/"),
                (
                    "menubar.report-issue",
                    "https://github.com/danirod/cartero/issues",
                ),
                (
                    "menubar.view-discussions",
                    "https://github.com/danirod/cartero/discussions",
                ),
                (
                    "menubar.translate",
                    "https://hosted.weblate.org/projects/cartero/cartero",
                ),
            ];

            let actions = links
                .into_iter()
                .map(|(id, url)| {
                    ActionEntryBuilder::new(id)
                        .activate(move |_, _, _| {
                            glib::spawn_future_local(async move {
                                let _ = gtk::UriLauncher::new(url)
                                    .launch_future(gtk::Window::NONE)
                                    .await;
                            });
                        })
                        .build()
                })
                .collect::<Vec<_>>();
            self.add_action_entries(actions);
        }
    }

    pub fn last_session_tabs(&self) -> Vec<gio::File> {
        self.imp()
            .settings
            .get::<Vec<String>>("open-files")
            .iter()
            .filter_map(|path| {
                // These paths are in the format file:///home/user/endpoint.cartero.
                // TODO: Can't use the url crate to extract the path from the scheme?
                if let Some((_type, path)) = path.split_once(':') {
                    let file = gio::File::for_path(path);
                    Some(file)
                } else {
                    None
                }
            })
            .collect()
    }

    #[cfg(feature = "app_updater")]
    async fn action_check_updates(&self) {
        use crate::updates::AppUpdateDialogResponse;

        let root = self.active_window().expect("No active window?");

        match crate::updates::get_latest_version().await {
            None => crate::updates::notify_check_update_error(&root).await,
            Some(response) => {
                use crate::config::VERSION;

                if !response.needs_update(VERSION) {
                    crate::updates::notify_latest_version(&root).await;
                } else {
                    match crate::updates::notify_app_available(
                        &root,
                        &response.get_latest_version(),
                    )
                    .await
                    {
                        AppUpdateDialogResponse::Skip => {}
                        AppUpdateDialogResponse::OpenWebsite => {
                            let _ = gtk::UriLauncher::new("https://cartero.danirod.es")
                                .launch_future(gtk::Window::NONE)
                                .await;
                        }
                    }
                }
            }
        }
    }

    pub fn new_window<T>(&self, child: &T) -> gtk::ApplicationWindow
    where
        T: IsA<gtk::Widget>,
    {
        if cfg!(feature = "csd") {
            let window = adw::ApplicationWindow::new(self);
            window.set_content(Some(child));
            window.upcast()
        } else {
            let window = gtk::ApplicationWindow::new(self);
            window.set_child(Some(child));
            window.upcast()
        }
    }
}
