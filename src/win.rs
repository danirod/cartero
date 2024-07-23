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

use crate::{app::CarteroApplication, error::CarteroError};
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::{gio, glib};

mod imp {
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    use adw::prelude::AlertDialogExtManual;
    use adw::{subclass::prelude::*, TabPage};
    use glib::property::PropertySet;
    use gtk::gio::ActionEntry;
    use gtk::{prelude::*, ExpressionWatch};

    use crate::{app::CarteroApplication, error::CarteroError};
    use crate::{config, widgets::*};
    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, TemplateChild};

    #[cfg(feature = "csd")]
    use gettextrs::gettext;

    #[cfg(feature = "csd")]
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/main_window.ui")]
    pub struct CarteroWindow {
        #[template_child]
        toaster: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub tabs: TemplateChild<adw::TabBar>,

        #[template_child]
        pub tabview: TemplateChild<adw::TabView>,

        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,

        window_title_binding: RefCell<Option<ExpressionWatch>>,
    }

    #[cfg(not(feature = "csd"))]
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/main_window_no_csd.ui")]
    pub struct CarteroWindow {
        #[template_child]
        toaster: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub tabs: TemplateChild<adw::TabBar>,

        #[template_child]
        pub tabview: TemplateChild<adw::TabView>,

        window_title_binding: RefCell<Option<ExpressionWatch>>,
    }

    #[gtk::template_callbacks]
    impl CarteroWindow {
        #[cfg(feature = "csd")]
        fn bind_current_tab(&self, tab: &ItemPane) {
            {
                let value: &Option<ExpressionWatch> = &self.window_title_binding.borrow();
                if let Some(binding) = value {
                    binding.unwatch();
                }
            }

            let new_binding =
                tab.window_title_binding()
                    .bind(&*self.window_title, "title", Some(tab));
            let subtitle = tab.path().unwrap_or(gettext("Draft"));
            self.window_title.set_subtitle(&subtitle);
            self.window_title_binding.set(Some(new_binding));
        }

        #[cfg(not(feature = "csd"))]
        fn bind_current_tab(&self, tab: &ItemPane) {
            {
                let value: &Option<ExpressionWatch> = &self.window_title_binding.borrow();
                if let Some(binding) = value {
                    binding.unwatch();
                }
            }

            let obj = self.obj();
            let title_watch = tab
                .window_title_binding()
                .chain_closure::<String>(glib::closure!(|_: &ItemPane, title: &str| {
                    format!("{title} - Cartero")
                }))
                .bind(&*obj, "title", Some(tab));
            self.window_title_binding.set(Some(title_watch));
        }

        fn init_settings(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();
            let obj = self.obj();

            let actions = [
                "auto-indent",
                "body-wrap",
                "indent-style",
                "show-line-numbers",
                "tab-width",
            ];
            for action in actions {
                let action = settings.create_action(action);
                obj.add_action(&action);
            }
            settings
                .bind("window-width", &*obj, "default-width")
                .build();
            settings
                .bind("window-height", &*obj, "default-height")
                .build();
        }

        pub fn save_visible_tabs(&self) {
            let pages = self.tabview.pages();
            let count = pages.n_items();
            let mut paths = Vec::new();
            for i in 0..count {
                let page = pages.item(i).and_downcast::<TabPage>().unwrap();
                let child = page.child().downcast::<ItemPane>().unwrap();
                let path = child.path();
                if let Some(path) = path {
                    let path = format!("endpoint:{path}");
                    paths.push(path);
                }
            }

            let app = CarteroApplication::get();
            let settings = app.settings();
            settings.set("open-files", paths).unwrap();
        }

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
                    pane.window_title_binding()
                        .bind(&page, "title", Some(&pane));
                    pane.bind_property("path", &page, "tooltip")
                        .sync_create()
                        .build();
                    self.tabview.set_selected_page(&page);
                    self.save_visible_tabs();
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
                self.save_visible_tabs();
            }
            Ok(())
        }

        async fn save_pane(&self, pane: &ItemPane) -> Result<(), CarteroError> {
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
                let endpoint = endpoint.extract_endpoint()?;
                let serialized_payload = crate::file::store_toml(&endpoint)?;
                crate::file::write_file(&path, &serialized_payload)?;
                pane.update_title_and_path(&path);
                pane.set_dirty(false);
                self.bind_current_tab(pane);
                self.save_visible_tabs();
            }

            Ok(())
        }

        async fn trigger_save(&self) -> Result<(), CarteroError> {
            // In order to place the modal, we need a reference to the public type.
            let Some(pane) = self.current_pane() else {
                return Ok(());
            };
            self.save_pane(&pane).await
        }

        pub(super) fn toast_error(&self, error: CarteroError) {
            let toast = adw::Toast::new(&error.to_string());
            self.toaster.add_toast(toast);
        }

        fn get_modified_panes(&self) -> Vec<ItemPane> {
            let pages = self.tabview.pages();
            let count = pages.n_items();
            let mut panes = Vec::new();

            for i in 0..count {
                let page = pages.item(i).and_downcast::<TabPage>().unwrap();
                let child = page.child().downcast::<ItemPane>().unwrap();
                if child.dirty() {
                    panes.push(child.clone());
                }
            }

            panes
        }

        async fn show_save_changes(&self) -> String {
            let app = CarteroApplication::get();
            let window = app.get_window();
            let dialog = SaveDialog::default();
            dialog.choose_future(window).await.as_str().to_string()
        }

        async fn save_all_tabs(&self) -> Result<(), CarteroError> {
            let panes = self.get_modified_panes();
            for pane in panes {
                self.save_pane(&pane).await?
            }
            Ok(())
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroWindow {
        const NAME: &'static str = "CarteroWindow";
        type Type = super::CarteroWindow;

        #[cfg(feature = "csd")]
        type ParentType = adw::ApplicationWindow;

        #[cfg(not(feature = "csd"))]
        type ParentType = gtk::ApplicationWindow;

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

            if config::PROFILE == "Devel" {
                let obj = self.obj();
                obj.add_css_class("devel");
            }

            self.init_settings();

            self.tabview.connect_selected_page_notify(
                glib::clone!(@weak self as window => move |tabview| {
                    if let Some(page) = tabview.selected_page() {
                        let item_pane = page.child().downcast::<ItemPane>().unwrap();
                        window.bind_current_tab(&item_pane);
                    }
                }),
            );

            self.tabview.connect_close_page(glib::clone!(@weak self as window => @default-return true, move |tabview, tabpage| {
                let item_pane = tabpage.child().downcast::<ItemPane>().unwrap();
                let outcome = if item_pane.dirty() {
                    let app = CarteroApplication::get();
                    let win = app.get_window();
                    let dialog = SaveDialog::default();
                    let response = glib::MainContext::default().block_on(dialog.choose_future(win));
                    match response.as_str() {
                        "save" => {
                            let resp = glib::MainContext::default().block_on(window.save_pane(&item_pane));
                            match resp {
                                Ok(_) => false,
                                Err(e) => {
                                    window.toast_error(e);
                                    true
                                },
                            }
                        },
                        "discard" => false,
                        _ => true,


                    }
                } else {
                    item_pane.set_path(Option::<String>::None);
                    let app = CarteroApplication::get();
                    app.get_window().sync_open_files();
                    false
                };

                tabview.close_page_finish(tabpage, !outcome);
                true
            }));

            self.tabview.connect_page_reordered(
                glib::clone!(@weak self as window => move |_, _, _| {
                        window.save_visible_tabs();
                    }
                ),
            );

            let action_new = ActionEntry::builder("new")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    window.add_endpoint(None);
                }))
                .build();

            let action_request = ActionEntry::builder("request")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Some(pane) = window.current_pane().and_then(|e| e.endpoint()) {
                            pane.set_sensitive(false);
                            if let Err(e) = pane.perform_request().await {
                                window.toast_error(e);
                            }
                            pane.set_sensitive(true);
                        }
                    }));
                }))
                .build();
            let action_open = ActionEntry::builder("open")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_open().await {
                            window.toast_error(e);
                        }
                    }));
                }))
                .build();
            let action_save = ActionEntry::builder("save")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_save().await {
                            window.toast_error(e);
                        }
                    }));
                }))
                .build();

            let obj = self.obj();
            obj.add_action_entries([action_new, action_request, action_open, action_save]);
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {
        fn close_request(&self) -> glib::Propagation {
            let panes = self.get_modified_panes();
            if panes.is_empty() {
                glib::Propagation::Proceed
            } else {
                let response = glib::MainContext::default().block_on(self.show_save_changes());
                match response.as_str() {
                    "discard" => glib::Propagation::Proceed,
                    "save" => {
                        let result = glib::MainContext::default().block_on(self.save_all_tabs());
                        match result {
                            Ok(_) => glib::Propagation::Proceed,
                            Err(e) => {
                                self.toast_error(e);
                                glib::Propagation::Stop
                            }
                        }
                    }
                    _ => glib::Propagation::Stop,
                }
            }
        }
    }

    impl ApplicationWindowImpl for CarteroWindow {}

    #[cfg(feature = "csd")]
    impl AdwApplicationWindowImpl for CarteroWindow {}
}

#[cfg(feature = "csd")]
glib::wrapper! {
    pub struct CarteroWindow(ObjectSubclass<imp::CarteroWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Root;
}

#[cfg(not(feature = "csd"))]
glib::wrapper! {
    pub struct CarteroWindow(ObjectSubclass<imp::CarteroWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Root;
}

impl CarteroWindow {
    pub fn new(app: &CarteroApplication) -> Self {
        Object::builder().property("application", Some(app)).build()
    }

    pub fn add_endpoint(&self, ep: Option<&PathBuf>) {
        let imp = &self.imp();
        imp.add_endpoint(ep)
    }

    pub fn toast_error(&self, e: CarteroError) {
        let imp = self.imp();
        imp.toast_error(e);
    }

    pub fn sync_open_files(&self) {
        let imp = self.imp();
        imp.save_visible_tabs();
    }
}
