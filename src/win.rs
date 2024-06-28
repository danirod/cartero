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

use std::path::PathBuf;

use crate::app::CarteroApplication;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::{
    gio::{self, Settings},
    glib,
    prelude::SettingsExtManual,
};

use gtk::prelude::ActionMapExt;
use gtk::prelude::SettingsExt;

mod imp {

    use std::path::{Path, PathBuf};

    use adw::{subclass::prelude::*, TabPage};
    use gtk::gio::ActionEntry;
    use gtk::prelude::*;

    use crate::error::CarteroError;
    use crate::widgets::*;
    use glib::subclass::InitializingObject;
    use gtk::{
        subclass::{
            application_window::ApplicationWindowImpl, widget::WidgetImpl, window::WindowImpl,
        },
        CompositeTemplate, TemplateChild,
    };

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/main_window.ui")]
    pub struct CarteroWindow {
        #[template_child]
        pub tabs: TemplateChild<adw::TabBar>,

        #[template_child]
        pub tabview: TemplateChild<adw::TabView>,

        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
    }

    #[gtk::template_callbacks]
    impl CarteroWindow {
        /// Returns the pane currently visible in the window.
        ///
        /// This method will make more sense in the future once multiple panes can be visible in tabs.
        pub fn current_pane(&self) -> Option<ItemPane> {
            let page = self.tabview.selected_page()?;
            let page = page.child().downcast::<ItemPane>().unwrap();
            Some(page)
        }

        fn find_pane_by_path(&self, path: &Path) -> Option<TabPage> {
            let path_str = path.to_str().unwrap();
            self.tabview
                .pages()
                .iter::<TabPage>()
                .filter(Result::is_ok)
                .flatten()
                .find(|page| {
                    let item = page.child().downcast::<ItemPane>().unwrap();
                    let item_path = item.path();
                    item_path.is_some_and(|p| p == path_str)
                })
        }

        pub fn add_endpoint(&self, path: Option<&PathBuf>) {
            if let Some(path) = path {
                if let Some(tab) = self.find_pane_by_path(path) {
                    self.tabview.set_selected_page(&tab);
                    return;
                }
            }

            match ItemPane::new_for_endpoint(path) {
                Ok(pane) => {
                    let page = self.tabview.add_page(&pane, None);
                    pane.bind_property("title", &page, "title")
                        .sync_create()
                        .build();
                    pane.bind_property("path", &page, "tooltip")
                        .sync_create()
                        .build();
                    self.tabview.set_selected_page(&page);
                }
                Err(e) => {
                    println!("TODO: Show global error -- {}", e);
                }
            };
        }

        async fn trigger_open(&self) -> Result<(), CarteroError> {
            // In order to place the modal, we need a reference to the public type.
            let obj = self.obj();
            let path = crate::widgets::open_file(&obj).await?;
            if path.is_some() {
                self.add_endpoint(path.as_ref());
            }
            Ok(())
        }

        async fn trigger_save(&self) -> Result<(), CarteroError> {
            // In order to place the modal, we need a reference to the public type.
            let Some(pane) = self.current_pane() else {
                return Ok(());
            };

            let Some(endpoint) = pane.endpoint() else {
                return Ok(());
            };

            let path = match pane.path() {
                Some(path) => Some(PathBuf::from(path)),
                None => {
                    let obj = self.obj();
                    crate::widgets::save_file(&obj).await?
                }
            };

            if let Some(path) = path {
                println!("Saving as {:?}", path);

                let endpoint = endpoint.extract_endpoint()?;
                let serialized_payload = crate::file::store_toml(endpoint)?;
                crate::file::write_file(&path, &serialized_payload)?;
                pane.update_title_and_path(&path);

                self.window_title.set_title(&pane.title());
                self.window_title
                    .set_subtitle(&pane.path().unwrap_or("Draft".into()));
            }

            Ok(())
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroWindow {
        const NAME: &'static str = "CarteroWindow";
        type Type = super::CarteroWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            KeyValueRow::static_type();
            KeyValuePane::static_type();
            EndpointPane::static_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CarteroWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.tabview.connect_selected_page_notify(
                glib::clone!(@weak self as window => move |tabview| {
                    if let Some(page) = tabview.selected_page() {
                        let item_pane = page.child().downcast::<ItemPane>().unwrap();
                        window.window_title.set_title(&item_pane.title());
                        window.window_title.set_subtitle(&item_pane.path().unwrap_or("Draft".into()));
                    }
                }),
            );

            let action_new = ActionEntry::builder("new")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    window.add_endpoint(None);
                }))
                .build();

            let action_request = ActionEntry::builder("request")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    let Some(pane) = window.current_pane() else {
                        println!("No request");
                        return;
                    };

                    let Some(pane) = pane.endpoint() else {
                        println!("Not a request");
                        return;
                    };

                    if let Err(e) = pane.perform_request() {
                        let error_msg = format!("{}", e);
                        pane.show_banner(&error_msg);
                    }
                }))
                .build();
            let action_open = ActionEntry::builder("open")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_open().await {
                            let error_msg = format!("{}", e);
                            println!("{:?}", error_msg);
                        }
                    }));
                }))
                .build();
            let action_save = ActionEntry::builder("save")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_save().await {
                            let error_msg = format!("{}", e);
                            println!("{:?}", error_msg);
                        }
                    }));
                }))
                .build();

            let obj = self.obj();
            obj.add_action_entries([action_new, action_request, action_open, action_save]);
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {}

    impl ApplicationWindowImpl for CarteroWindow {}

    impl AdwApplicationWindowImpl for CarteroWindow {}
}

glib::wrapper! {
    pub struct CarteroWindow(ObjectSubclass<imp::CarteroWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Root;
}

impl CarteroWindow {
    pub fn new(app: &CarteroApplication) -> Self {
        Object::builder().property("application", Some(app)).build()
    }

    pub fn assign_settings(&self, settings: &Settings) {
        let wrap = settings.create_action("body-wrap");
        self.add_action(&wrap);

        settings.bind("window-width", self, "default-width").build();
        settings
            .bind("window-height", self, "default-height")
            .build();
    }

    pub fn show_banner(&self, str: &str) {
        let imp = &self.imp();
        if let Some(pane) = imp.current_pane() {
            let Some(pane) = pane.endpoint() else {
                println!("Not a request");
                return;
            };
            pane.show_banner(str);
        }
    }

    pub fn hide_banner(&self) {
        let imp = &self.imp();
        if let Some(pane) = imp.current_pane() {
            let Some(pane) = pane.endpoint() else {
                println!("Not a request");
                return;
            };
            pane.hide_banner();
        }
    }

    pub fn add_endpoint(&self, ep: Option<&PathBuf>) {
        let imp = &self.imp();
        imp.add_endpoint(ep)
    }
}
