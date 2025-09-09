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
use adw::subclass::prelude::*;

use crate::windows::settings::pill::Pill;

glib::wrapper! {
    pub struct Shell(ObjectSubclass<imp::Shell>)
        @extends gtk::Widget, adw::BreakpointBin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Shell {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn init_native_window(&self, win: &gtk::Window) {
        self.imp().init_native_window(win);
    }

    pub fn set_page(&self, page: &str) {
        let mut index = 0;
        while let Some(row) = self.imp().sidebar_box.row_at_index(index) {
            let child = row.child().and_downcast::<Pill>().expect("Not a pill?");
            if child.name() == page {
                row.activate();
                return;
            }
            index = index + 1;
        }
    }
}

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;

    use crate::windows::{
        common::CommonShell,
        settings::{pages, pill::Pill},
    };

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/shell.ui")]
    pub struct Shell {
        #[template_child]
        split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        settings_page: TemplateChild<adw::NavigationPage>,
        #[template_child]
        sidebar: TemplateChild<adw::NavigationPage>,
        #[template_child]
        settings_header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        sidebar_header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(super) settings_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub(super) sidebar_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Shell {
        const NAME: &'static str = "CarteroSettingsShell";
        type Type = super::Shell;
        type ParentType = adw::BreakpointBin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.add_binding_action(
                gtk::gdk::Key::Escape,
                gtk::gdk::ModifierType::empty(),
                "window.close",
            );
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Shell {
        fn constructed(&self) {
            self.parent_constructed();

            self.init_content_view();
            self.init_sidebar();
            self.init_root_window();
            self.obj()
                .connect_root_notify(|shell| shell.imp().init_root_window());

            self.update_settings_page_title();
            self.settings_stack
                .connect_visible_child_notify(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.update_settings_page_title();
                    }
                ));
        }
    }

    impl WidgetImpl for Shell {}

    impl BreakpointBinImpl for Shell {}

    impl CommonShell for Shell {}

    #[gtk::template_callbacks]
    impl Shell {
        #[template_callback]
        fn on_check_updates() {}

        #[template_callback]
        fn on_row_selected(&self, row: Option<&gtk::ListBoxRow>) {
            if let Some(pill) = row.and_then(|row| row.child()).and_downcast::<Pill>() {
                let target = pill.name();
                self.settings_stack.set_visible_child_name(&target);
                self.split_view.set_show_content(true);
            }
        }

        pub(super) fn init_native_window(&self, win: &gtk::Window) {
            crate::native::prepare_window(&win);

            win.connect_fullscreened_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |win| {
                    imp.update_native_appearance(&win);
                }
            ));
            win.connect_default_width_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |win| {
                    imp.update_native_appearance(&win);
                }
            ));
            win.connect_default_height_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |win| {
                    imp.update_native_appearance(&win);
                }
            ));
            win.connect_realize(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |win| {
                    imp.update_native_appearance(&win);
                }
            ));
        }

        fn update_native_appearance(&self, win: &gtk::Window) {
            let sidebar_width = 175;
            crate::native::update_vibrancy(&win, None, Some(sidebar_width));
        }

        fn init_root_window(&self) {
            self.sidebar_header_bar.set_visible(self.is_client_side());
            self.settings_header_bar
                .set_show_start_title_buttons(self.is_client_side());
            self.settings_header_bar
                .set_show_end_title_buttons(self.is_client_side());
        }

        fn init_content_view(&self) {
            if adw::major_version() == 1 && adw::minor_version() >= 7 {
                self.settings_stack.set_property("enable-transitions", true);
            }

            let pages: Vec<adw::PreferencesPage> = vec![
                pages::Application::new().upcast(),
                pages::Security::new().upcast(),
                pages::Appearance::new().upcast(),
                pages::CodeEditor::new().upcast(),
                pages::HttpClient::new().upcast(),
                pages::Proxy::new().upcast(),
            ];

            for page in pages {
                let child = page;
                let name = child.name().map(|s| s.to_string());
                let title = child.title();
                let icon_name = child.icon_name().map(|s| s.to_string()).unwrap_or_default();
                self.settings_stack.add_titled_with_icon(
                    &child,
                    name.as_deref(),
                    title.as_str(),
                    icon_name.as_str(),
                );
            }
        }

        fn init_sidebar(&self) {
            for page in self.settings_stack.pages().iter::<adw::ViewStackPage>() {
                if let Ok(page) = page {
                    let pill: Pill = glib::Object::builder()
                        .property("icon-name", page.icon_name())
                        .property("label", page.title())
                        .property("name", page.name())
                        .build();

                    let child = gtk::ListBoxRow::builder().child(&pill).build();
                    self.sidebar_box.append(&child);
                }
            }
        }

        fn update_settings_page_title(&self) {
            let title_name = self
                .settings_stack
                .visible_child()
                .map(|page| self.settings_stack.page(&page))
                .and_then(|page| page.title())
                .unwrap_or_default();
            self.settings_page.set_title(title_name.as_str());
        }
    }
}
