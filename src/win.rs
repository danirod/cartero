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
use gtk::prelude::*;
use glib::Object;
use gtk::{gio, glib};

mod imp {
    use std::path::PathBuf;
    use std::path::Path;

    use adw::prelude::*;
    use adw::{subclass::prelude::*, TabPage};
    use glib::closure_local;
    use gtk::gio::ActionEntry;
    use gtk::gio::File;

    use crate::fs::collection::open_collection;
    use crate::objects::Collection;
    use crate::widgets::*;
    use crate::{app::CarteroApplication, error::CarteroError};
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
        toaster: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub tabs: TemplateChild<adw::TabBar>,

        #[template_child]
        pub tabview: TemplateChild<adw::TabView>,

        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        
        #[template_child]
        pub collections: TemplateChild<Sidebar>,
    }

    #[gtk::template_callbacks]
    impl CarteroWindow {
        pub(super) fn toast_error(&self, error: CarteroError) {
            let toast = adw::Toast::new(&error.to_string());
            self.toaster.add_toast(toast);
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
                Ok(_) => {
                    self.finish_open_collection(&path)
                }
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
            self.init_actions();
            self.init_settings();
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
        let win: CarteroWindow = Object::builder().property("application", Some(app)).build();

        let imp = win.imp();
        imp.finish_init();

        win
    }

    pub fn add_endpoint(&self, ep: Option<&PathBuf>) {
        let imp = &self.imp();
        imp.add_endpoint(ep)
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
}
