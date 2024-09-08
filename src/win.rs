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

use crate::{app::CarteroApplication, error::CarteroError, objects::Collection};
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::prelude::*;
use gtk::{gio, glib, prelude::SettingsExtManual};

mod imp {
    use std::path::Path;
    use std::path::PathBuf;

    use adw::prelude::AlertDialogExtManual;
    use adw::prelude::*;
    use adw::AboutWindow;
    use adw::{subclass::prelude::*, TabPage};
    use gettextrs::gettext;
    use glib::closure_local;
    use gtk::gio::File;
    use gtk::gio::{self, ActionEntry};
    use gtk::prelude::*;

    use crate::fs::collection::open_collection;
    use crate::objects::Collection;
    use crate::utils::SingleExpressionWatch;
    use crate::widgets::*;
    use crate::{app::CarteroApplication, error::CarteroError};
    use crate::{config, widgets::*};
    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, TemplateChild};

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

        #[template_child]
        pub collections: TemplateChild<Sidebar>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        window_title_binding: SingleExpressionWatch,

        window_subtitle_binding: SingleExpressionWatch,
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

        #[template_child]
        stack: TemplateChild<gtk::Stack>,
    }

    #[gtk::template_callbacks]
    impl CarteroWindow {
        fn update_tab_actions(&self) {
            let has_tabs = self.tabview.n_pages() > 0;
            let obj = self.obj();
            let actions = vec!["save", "save-as", "close"];
            for action in actions {
                if let Some(action) = obj.lookup_action(action) {
                    action.set_property("enabled", has_tabs);
                }
            }
        }

        #[cfg(feature = "csd")]
        fn bind_current_tab(&self, tab: Option<&ItemPane>) {
            self.window_title_binding.clear();
            self.window_subtitle_binding.clear();
            match tab {
                Some(tab) => {
                    let title_bind =
                        tab.window_title_binding()
                            .bind(&*self.window_title, "title", Some(tab));
                    let subtitle_bind = tab.window_subtitle_binding().bind(
                        &*self.window_title,
                        "subtitle",
                        Some(tab),
                    );
                    self.window_title_binding.replace(title_bind);
                    self.window_subtitle_binding.replace(subtitle_bind);
                }
                None => {
                    self.window_title.set_title("Cartero");
                    self.window_title.set_subtitle("");
                    self.window_title_binding.clear();
                    self.window_subtitle_binding.clear();
                }
            };
        }

        #[cfg(not(feature = "csd"))]
        fn bind_current_tab(&self, _: Option<&ItemPane>) {}

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

            // The following settings are only read once. They will be saved when the window closes.
            let width = settings.get::<i32>("window-width");
            let height = settings.get::<i32>("window-height");
            let maximized = settings.get::<bool>("is-maximized");
            obj.set_default_width(width);
            obj.set_default_height(height);
            obj.set_maximized(maximized);
        }

        fn save_window_state(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();
            let obj = self.obj();

            let _ = settings.set("window-width", obj.width());
            let _ = settings.set("window-height", obj.height());
            let _ = settings.set("is-maximized", obj.is_maximized());
        }

        fn finish_window_close(&self) -> glib::Propagation {
            self.save_window_state();
            glib::Propagation::Proceed
        }

        pub fn save_visible_tabs(&self) {
            let pages = self.tabview.pages();
            let count = pages.n_items();
            let mut paths = Vec::new();
            for i in 0..count {
                let page = pages.item(i).and_downcast::<TabPage>().unwrap();
                let child = page.child().downcast::<ItemPane>().unwrap();
                let path = child.file();
                let file = path
                    .and_then(|f| f.path())
                    .map(|pb| pb.display().to_string());
                if let Some(path) = file {
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

        fn find_pane_by_path(&self, file: &gio::File) -> Option<TabPage> {
            self.tabview
                .pages()
                .iter::<TabPage>()
                .filter(Result::is_ok)
                .flatten()
                .find(|page| {
                    let item = page.child().downcast::<ItemPane>().unwrap();
                    match item.file() {
                        Some(f) => f.equal(file),
                        None => false,
                    }
                })
        }

        pub async fn add_endpoint(&self, file: Option<&gio::File>) {
            if let Some(file) = file {
                if let Some(tab) = self.find_pane_by_path(file) {
                    self.tabview.set_selected_page(&tab);
                    return;
                }
            }

            /* If the current tab is a new document, replace it. */
            if let Some(pane) = self.current_pane() {
                if file.is_some() && !pane.dirty() && pane.file().is_none() {
                    let tp = self.tabview.page(&pane);
                    self.tabview.close_page(&tp);
                }
            }

            match ItemPane::new_for_endpoint(file).await {
                Ok(pane) => {
                    self.stack.set_visible_child_name("tabview");
                    let page = self.tabview.add_page(&pane, None);
                    pane.window_title_binding()
                        .bind(&page, "title", Some(&pane));
                    pane.window_subtitle_binding()
                        .bind(&page, "tooltip", Some(&pane));
                    self.tabview.set_selected_page(&page);
                    self.save_visible_tabs();
                }
                Err(e) => {
                    self.obj().toast_error(e);
                }
            };
        }

        pub fn open_collection_pane(&self, collection: &Collection) {
            let pane = CollectionPane::default();

            pane.load_collection(collection);
            let saved_collection = collection.downgrade();
            pane.connect_closure(
                "save-requested",
                false,
                closure_local!(move |pane: CollectionPane| {
                    let collection = saved_collection.upgrade().unwrap();
                    pane.save_collection(&collection);
                }),
            );

            let page = self.tabview.add_page(&pane, None);
            collection
                .bind_property("name", &page, "title")
                .sync_create()
                .build();
            self.tabview.set_selected_page(&page);
        }

        async fn trigger_new_collection(&self) -> Result<(), CarteroError> {
            let app = CarteroApplication::get();
            let settings = app.settings();

            let window = &*self.obj();
            let dialog = NewCollectionWindow::new();
            let last_dir = settings
                .get::<Option<String>>("last-directory-new-collection")
                .unwrap_or("~/".into());
            dialog.set_initial_directory(&last_dir);
            dialog.present(window);
            Ok(())
        }

        async fn trigger_open(&self) -> Result<(), CarteroError> {
            // In order to place the modal, we need a reference to the public type.
            let obj = self.obj();
            let paths = crate::widgets::open_files(&obj).await?;
            for path in paths {
                self.add_endpoint(Some(&path)).await;
            }
            self.save_visible_tabs();
            Ok(())
        }

        async fn save_pane(&self, pane: &ItemPane) -> Result<(), CarteroError> {
            let Some(endpoint) = pane.endpoint() else {
                return Ok(());
            };

            let file = match pane.file() {
                Some(file) => file,
                None => {
                    let obj = self.obj();
                    crate::widgets::save_file(&obj).await?
                }
            };

            let endpoint = endpoint.extract_endpoint()?;
            let serialized_payload = crate::file::store_toml(&endpoint)?;
            crate::file::write_file(&file, &serialized_payload).await?;
            pane.set_file(Some(file.clone()));
            pane.set_dirty(false);

            Ok(())
        }

        async fn trigger_open_collection(&self) -> Result<(), CarteroError> {
            // Get last directory from settings
            let app = CarteroApplication::get();
            let settings = app.settings();
            let last_dir = settings
                .get::<Option<String>>("last-directory-open-collection")
                .unwrap_or("~/".into());
            let last_dir_path = File::for_path(last_dir);

            // Request for the collection directory.
            let dialog = gtk::FileDialog::new();
            dialog.set_initial_folder(Some(&last_dir_path));
            let window = &*self.obj();
            let Ok(result) = dialog.select_folder_future(Some(window)).await else {
                return Err(CarteroError::FileDialogError);
            };

            // Save this as the most recent directory for the last-directory-open-collection
            let path = result.path().unwrap();
            let parent_dir = path.parent().and_then(Path::to_str);
            let _ = settings.set("last-directory-open-collection", parent_dir);

            match open_collection(&path) {
                Ok(_) => self.finish_open_collection(&path),
                Err(e) => Err(e),
            }
        }

        /// Call this function to actually open a collection and add it to the sidebar
        /// and to the recents list. Call to this function should be done always.
        /// If you create a collection, just pass a pointer to the newly created
        /// collection. If you open a collection, pass a pointer to the collection
        /// that you are opening.
        pub fn finish_open_collection(&self, path: &Path) -> Result<(), CarteroError> {
            let app = CarteroApplication::get();
            let settings = app.settings();

            // Update the open collections list.
            let mut value: Vec<String> = settings.get("open-collections");
            let Some(new_path) = path.to_str() else {
                return Err(CarteroError::FileDialogError);
            };
            let new_path_str = new_path.to_string();

            // Make sure that the collection is not already opened
            let already_in = value.iter().any(|s| s == &new_path_str);
            if already_in {
                return Err(CarteroError::AlreadyOpened);
            }

            // Everything is fine
            value.push(new_path_str);
            settings
                .set("open-collections", value)
                .map_err(|_| CarteroError::FileDialogError)?;

            // Finally, update the sidebar and close the dialog
            self.collections.sync_collections(&settings);

            Ok(())
        }

        pub fn finish_create_collection(&self, path: &PathBuf) -> Result<(), CarteroError> {
            let app = CarteroApplication::get();
            let settings = app.settings();

            if let Some(new_initial_dir) = path.parent() {
                let path_str = new_initial_dir.to_str();
                let _ = settings.set("last-directory-new-collection", path_str);
            }

            let collection = Collection::new();
            crate::fs::collection::save_collection(path, &collection)?;

            self.finish_open_collection(path)
        }

        fn init_sidebar(&self) {
            let obj = self.obj();
            let application = obj
                .application()
                .and_downcast::<CarteroApplication>()
                .unwrap();
            let settings = application.settings();

            self.collections.sync_collections(settings);
        }

        fn init_actions(&self) {
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

            let action_new_collection = ActionEntry::builder("new-collection")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_new_collection().await {
                            window.toast_error(e);
                        }
                    }));
                }))
                .build();

            let action_open_collection = ActionEntry::builder("open-collection")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_open_collection().await {
                            window.toast_error(e);
                        }
                    }));
                }))
                .build();

            let obj = self.obj();
            obj.add_action_entries([
                action_new,
                action_request,
                action_new_collection,
                action_open_collection,
            ]);
        }

        pub(super) fn finish_init(&self) {
            self.init_sidebar();
        }

        async fn save_pane_as(&self, pane: &ItemPane) -> Result<(), CarteroError> {
            let Some(endpoint) = pane.endpoint() else {
                return Ok(());
            };

            let obj = self.obj();
            let file = crate::widgets::save_file(&obj).await?;

            let endpoint = endpoint.extract_endpoint()?;
            let serialized_payload = crate::file::store_toml(&endpoint)?;
            crate::file::write_file(&file, &serialized_payload).await?;
            pane.set_file(Some(file.clone()));
            pane.set_dirty(false);

            Ok(())
        }

        async fn trigger_save(&self) -> Result<(), CarteroError> {
            let Some(pane) = self.current_pane() else {
                return Ok(());
            };
            let res = self.save_pane(&pane).await;
            if res.is_ok() {
                self.bind_current_tab(Some(&pane));
                self.save_visible_tabs();
            }
            res
        }

        async fn trigger_save_as(&self) -> Result<(), CarteroError> {
            let Some(pane) = self.current_pane() else {
                return Ok(());
            };
            let res = self.save_pane_as(&pane).await;
            if res.is_ok() {
                self.bind_current_tab(Some(&pane));
                self.save_visible_tabs();
            }
            res
        }

        pub(super) fn toast_error(&self, error: CarteroError) {
            let toast: adw::Toast = adw::Toast::new(&error.to_string());
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
            let window = self.obj();
            let dialog = SaveDialog::default();
            dialog.choose_future(&*window).await.as_str().to_string()
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
            self.init_actions();
            self.init_settings();

            if config::PROFILE == "Devel" {
                let obj = self.obj();
                obj.add_css_class("devel");
            }

            self.tabview.connect_selected_page_notify(
                glib::clone!(@weak self as window => move |tabview| {
                    if let Some(page) = tabview.selected_page() {
                        let item_pane = page.child().downcast::<ItemPane>().unwrap();
                        window.bind_current_tab(Some(&item_pane));
                        window.update_tab_actions();
                    }
                }),
            );

            let obj = self.obj();
            self.tabview.connect_close_page(glib::clone!(@weak obj as window => @default-return true, move |tabview, tabpage| {
                let item_pane = tabpage.child().downcast::<ItemPane>().unwrap();
                let imp = window.imp();
                let outcome = if item_pane.dirty() {
                    let dialog = SaveDialog::default();
                    let response = glib::MainContext::default().block_on(dialog.choose_future(&window));
                    match response.as_str() {
                        "save" => {
                            let resp = glib::MainContext::default().block_on(imp.save_pane(&item_pane));
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
                    item_pane.set_file(Option::<gio::File>::None);
                    window.sync_open_files();
                    false
                };

                tabview.close_page_finish(tabpage, !outcome);
                let imp = window.imp();
                imp.update_tab_actions();
                if imp.tabview.n_pages() == 0 {
                    imp.bind_current_tab(None);
                    imp.stack.set_visible_child_name("welcome");
                }
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
                    glib::spawn_future_local(async move {
                        window.add_endpoint(None).await;
                    });
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
                            match e {
                                CarteroError::NoFilePicked => {},
                                e => window.toast_error(e),
                            };
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
            let action_save_as = ActionEntry::builder("save-as")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_save_as().await {
                            window.toast_error(e);
                        }
                    }));
                }))
                .build();
            let action_close = ActionEntry::builder("close")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    if let Some(page) = window.tabview.selected_page() {
                        window.tabview.close_page(&page);
                    }
                }))
                .build();

            let action_about = ActionEntry::builder("about")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    let about = AboutWindow::builder()
                        .transient_for(&*window.obj())
                        .modal(true)
                        .application_name("Cartero")
                        .application_icon(config::APP_ID)
                        .version(config::VERSION)
                        .website("https://github.com/danirod/cartero")
                        .issue_url("https://github.com/danirod/cartero/issues")
                        .support_url("https://github.com/danirod/cartero/discussions")
                        .developer_name(gettext("The Cartero authors"))
                        .copyright(gettext("© 2024 the Cartero authors"))
                        .license_type(gtk::License::Gpl30)
                        .build();
                    about.present();
                }))
                .build();

            let obj = self.obj();
            obj.add_action_entries([
                action_new,
                action_request,
                action_open,
                action_save,
                action_save_as,
                action_close,
                action_about,
            ]);
            self.update_tab_actions();
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {
        fn close_request(&self) -> glib::Propagation {
            let panes = self.get_modified_panes();
            if panes.is_empty() {
                self.finish_window_close()
            } else {
                let response = glib::MainContext::default().block_on(self.show_save_changes());
                match response.as_str() {
                    "discard" => self.finish_window_close(),
                    "save" => {
                        let result = glib::MainContext::default().block_on(self.save_all_tabs());
                        match result {
                            Ok(_) => self.finish_window_close(),
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
        let win: CarteroWindow = Object::builder().property("application", Some(app)).build();

        let imp = win.imp();
        imp.finish_init();

        win
    }

    pub async fn add_endpoint(&self, ep: Option<&gio::File>) {
        let imp = &self.imp();
        imp.add_endpoint(ep).await
    }

    pub fn toast_error(&self, e: CarteroError) {
        let imp = self.imp();
        imp.toast_error(e);
    }

    pub fn open_collection_pane(&self, collection: &Collection) {
        let imp = &self.imp();
        imp.open_collection_pane(collection);
    }

    pub fn close_collection(&self, path: &str) {
        let imp = self.imp();

        let app = CarteroApplication::get();
        let settings = app.settings();

        let mut open_collections = settings.get::<Vec<String>>("open-collections");
        open_collections.retain(|p| p != path);
        let _ = settings.set("open-collections", open_collections);
        imp.collections.sync_collections(settings);
    }

    pub fn finish_create_collection(&self, path: &PathBuf) -> Result<(), CarteroError> {
        let imp = self.imp();
        imp.finish_create_collection(path)
    }

    pub fn sync_open_files(&self) {
        let imp = self.imp();
        imp.save_visible_tabs();
    }

    pub async fn open_last_session(&self) {
        let app = CarteroApplication::get();
        let settings = app.settings();
        let open_files = settings.get::<Vec<String>>("open-files");
        for open_file in open_files {
            let typed = open_file.split_once(':');
            if let Some((_type, path)) = typed {
                let path = gio::File::for_path(path);
                self.add_endpoint(Some(&path)).await;
            }
        }
    }
}
