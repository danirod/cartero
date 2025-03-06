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

use crate::{
    app::CarteroApplication, error::CarteroError, file::FileLoadFailure, widgets::ItemPane,
};
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::{gio, glib};
use indexmap::IndexMap;

mod imp {
    use std::collections::HashSet;

    use adw::prelude::WidgetExt;
    use adw::AboutWindow;
    use adw::{subclass::prelude::*, TabPage};
    use gettextrs::gettext;
    use gtk::gio::{self, ActionEntry};
    use gtk::prelude::*;
    use indexmap::IndexMap;

    use crate::error::FileSaveError;
    use crate::file::{FileLoadFailure, FileLoadResult};
    use crate::{app::CarteroApplication, error::CarteroError};
    use crate::{config, widgets::*};
    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, TemplateChild};

    #[derive(CompositeTemplate, Default)]
    #[cfg_attr(
        feature = "csd",
        template(resource = "/es/danirod/Cartero/main_window.ui")
    )]
    #[cfg_attr(
        not(feature = "csd"),
        template(resource = "/es/danirod/Cartero/main_window_no_csd.ui")
    )]
    pub struct CarteroWindow {
        #[template_child]
        toaster: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub tabs: TemplateChild<adw::TabBar>,

        #[template_child]
        pub tabview: TemplateChild<adw::TabView>,

        #[cfg(feature = "csd")]
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,
    }

    #[gtk::template_callbacks]
    impl CarteroWindow {
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

        fn insert_pane_into_tabs(&self, pane: &ItemPane) -> TabPage {
            // Wrap the pane into a page and mark it as the current one.
            let page = self.tabview.add_page(pane, None);
            self.stack.set_visible_child_name("tabview");
            self.tabview.set_selected_page(&page);

            // Define the bindings that act on the tab indicator.
            pane.window_title_binding().bind(&page, "title", Some(pane));
            pane.window_subtitle_binding()
                .bind(&page, "tooltip", Some(pane));

            page
        }

        /// This is the callback that gets called when the "New endpoint" action
        /// is triggered via the header bar button, the dropdown menu item or
        /// the key binding. Also triggered by the "New endpoint" button in the
        /// welcome screen.
        pub(super) fn action_new_endpoint(&self) {
            self.kill_clean_drafts();

            let pane = ItemPane::new_for_endpoint();
            self.insert_pane_into_tabs(&pane);
        }

        /// Returns a generic iterator to traverse the pages in the tab view.
        fn iter_pages(&self) -> impl Iterator<Item = TabPage> {
            self.tabview
                .pages()
                .snapshot()
                .into_iter()
                .map(|obj| obj.downcast::<adw::TabPage>().unwrap())
        }

        /// Returns a generic iterator to traverse the panes in the tab view.
        fn iter_panes(&self) -> impl Iterator<Item = ItemPane> {
            self.iter_pages()
                .map(|page| page.child().downcast::<ItemPane>().unwrap())
        }

        /// This function receives an iterator of gio::Files to open, and returns a filtered
        /// subset of these files where any file that is already opened has been removed.
        fn filter_endpoints_to_open(&self, endpoints: &[gio::File]) -> Vec<gio::File> {
            let opened = self
                .iter_panes()
                .filter_map(|pane| pane.file().map(|file| file.uri().to_string()))
                .collect::<HashSet<String>>();

            endpoints
                .into_iter()
                .filter_map(|file| {
                    if opened.contains(file.uri().as_str()) {
                        None
                    } else {
                        Some(file.clone())
                    }
                })
                .collect::<Vec<gio::File>>()
        }

        /// Given a list of endpoint files to open, this function will return the collection of
        /// ItemPanes that back the given endpoints. They will be loaded and the UI state will
        /// be populated, but they won't be added to the user interface yet.
        async fn preload_endpoints<I>(
            &self,
            files: I,
        ) -> IndexMap<ItemPane, Option<FileLoadFailure>>
        where
            I: IntoIterator<Item = gio::File> + Clone,
        {
            let mut loaded = IndexMap::new();
            for file in files {
                let pane = ItemPane::new_for_endpoint();
                pane.set_file(Some(file.clone()));
                let endpoint = pane.endpoint().expect("new_for_endpoint()?");
                let result = endpoint.load().await;
                loaded.insert(pane, result.failure());
            }
            loaded
        }

        async fn display_opened_panes(&self, panes: &IndexMap<ItemPane, Option<FileLoadFailure>>) {
            for (pane, failures) in panes {
                let can_open = match failures {
                    None => true,
                    Some(f) => f.error.is_none(),
                };
                if can_open {
                    self.insert_pane_into_tabs(&pane);
                }
            }
        }

        /// This is the callback that gets called when the "Open endpoint" action
        /// is triggered via the header bar button, the dropdown menu item or
        /// the key binding. Also triggered by the "Open endpoint" button in the
        /// welcome screen.
        async fn action_open_endpoint(&self) {
            let all_paths = self.gracefully_prompt_open_files().await;
            let not_opened_paths = self.filter_endpoints_to_open(&all_paths);

            if not_opened_paths.is_empty() {
                /* Every requested file is opened. Just switch to one of the requested panes. */
                if let Some(path) = all_paths.first() {
                    if let Some(page) = self.find_pane_by_path(path) {
                        self.tabview.set_selected_page(&page);
                    }
                }
            } else {
                let results = self.preload_endpoints(not_opened_paths).await;

                /* Because this is interactive, we can do all the UI update right now. */
                self.display_opened_panes(&results).await;
                self.report_open_endpoints_errors(&results).await;
            }

            self.save_visible_tabs();
        }

        /// This is the function that can be called from the outside to open a few
        /// endpoints on the current window given the list of files to be opened.
        ///
        /// It behaves similar to action_open_endpoint(), but does not assume that
        /// the window is visible, thus does not report errors. Errors are returned
        /// in the return object so that they can be reported later via the
        /// report_open_endpoints_errors() function.
        pub(super) async fn open_endpoints(
            &self,
            files: &[gio::File],
        ) -> IndexMap<ItemPane, Option<FileLoadFailure>> {
            let not_opened_paths = self.filter_endpoints_to_open(files);
            if not_opened_paths.is_empty() {
                /* Every requested file is opened. Just switch to one of the requested panes. */
                if let Some(path) = files.first() {
                    if let Some(page) = self.find_pane_by_path(path) {
                        self.tabview.set_selected_page(&page);
                    }
                }
                IndexMap::new()
            } else {
                let results = self.preload_endpoints(not_opened_paths).await;
                self.display_opened_panes(&results).await;
                self.save_visible_tabs();
                results
            }
        }

        /// This is the companion function for open_endpoints() that can be used
        /// to display in a batch every error related to a previous load operation.
        /// The reason behind this function is to defer presenting the dialogs
        /// until the window is actually visible.
        pub(super) async fn report_open_endpoints_errors(
            &self,
            opened: &IndexMap<ItemPane, Option<FileLoadFailure>>,
        ) {
            let obj = self.obj();
            for (pane, failures) in opened {
                if let Some(failures) = failures {
                    if let Some(error) = &failures.error {
                        dialogs::file_load_error_dialog(&*obj, pane.file().as_ref(), &error).await;
                    } else if !failures.warnings.is_empty() {
                        if let Some(file) = pane.file() {
                            if let Some(page) = self.find_pane_by_path(&file) {
                                self.tabview.set_selected_page(&page);
                            }
                        }
                        dialogs::file_load_warning_dialog(
                            &*obj,
                            pane.file().as_ref(),
                            &failures.warnings,
                        )
                        .await;
                    }
                }
            }
        }

        /// Calling this function will cause the current pane to be closed if it's anonymous
        /// (as in, not saved nor backed to a file) and is not dirty, so it has never
        /// actually been interacted with. Mainly used to cleanup the tab bar when a request
        /// is opened.
        fn kill_clean_drafts(&self) {
            let tabs = self.tabview.pages().snapshot();
            for tab in tabs {
                let page = tab.downcast::<TabPage>().unwrap();
                let pane = page.child().downcast::<ItemPane>().unwrap();
                if pane.file().is_none() && !pane.dirty() {
                    self.tabview.close_page(&page);
                }
            }
        }

        /// This function shows the user a file dialog to choose files to open.
        /// It will handle any error and present the proper alert dialogs if
        /// the operation fails, but this only covers errors related to the file
        /// dialog itself. The returned files may still fail during reading.
        /// The error is swalloed because the user is already notified.
        async fn gracefully_prompt_open_files(&self) -> Vec<gio::File> {
            let obj = self.obj();
            match crate::widgets::open_files(&obj).await {
                Ok(paths) => paths,
                Err(e) => {
                    dialogs::glib_file_dialog_error(&*obj, &e).await;
                    Vec::new()
                }
            }
        }

        /// This function shows the user a file dialog to choose a save file.
        /// It will handle any error and present the proper alert dialogs if
        /// the operation fails, but this only covers errors related to the file
        /// dialog itself. The requested file may still fail to save later.
        /// The error is swallowed because the user is already notified.
        async fn gracefully_prompt_save_file(&self) -> Option<gio::File> {
            let obj = self.obj();
            match crate::widgets::save_file(&*obj).await {
                Ok(maybe_file) => maybe_file,
                Err(e) => {
                    dialogs::glib_file_dialog_error(&*obj, &e).await;
                    None
                }
            }
        }

        async fn save_pane(&self, pane: &ItemPane) -> Option<Result<(), FileSaveError>> {
            let endpoint = pane.endpoint().unwrap();

            /* If the pane is anonymous, give it a chance to have a file. */
            let target_file = match pane.file() {
                Some(file) => Some(file),
                None => self.gracefully_prompt_save_file().await,
            };

            /* If the pane is still anonymous, the user has dismissed or the operation has failed. */
            if target_file.is_some() {
                let previous = pane.file();
                pane.set_file(target_file.clone());
                let result = endpoint.save().await;
                if let Err(e) = &result {
                    dialogs::file_save_error(&*self.obj(), target_file.as_ref(), e.clone()).await;
                    pane.set_file(previous);
                }
                Some(result)
            } else {
                None
            }
        }

        async fn action_save_endpoint(&self) {
            if let Some(pane) = self.current_pane() {
                self.save_pane(&pane).await;
                self.save_visible_tabs();
            }
        }

        async fn action_save_endpoint_as(&self) {
            if let Some(pane) = self.current_pane() {
                /* Request a new path for this pane. */
                let maybe_path = self.gracefully_prompt_save_file().await;
                if let Some(path) = maybe_path {
                    let previous = pane.file();
                    pane.set_file(Some(path));
                    let saved = self.save_pane(&pane).await;
                    if saved.is_none() || saved.is_some_and(|r| r.is_err()) {
                        pane.set_file(previous);
                    }
                    self.save_visible_tabs();
                }
            }
        }

        async fn close_tab_requested(&self, tabpage: &TabPage) {
            let obj = self.obj();
            let item_pane = tabpage.child().downcast::<ItemPane>().unwrap();
            let close_page = if item_pane.dirty() {
                /* The window has been modified, so we ask the user what to do. */
                match dialogs::confirm_save(&*obj, item_pane.file().as_ref()).await {
                    dialogs::SaveAlertDialogResponse::Save => {
                        /* Try to save, close if successful */
                        match self.save_pane(&item_pane).await {
                            Some(Ok(())) => true,
                            _ => false,
                        }
                    }
                    dialogs::SaveAlertDialogResponse::Discard => true, /* Discards the modifications */
                    _ => false,                                        /* Cancel, I guess */
                }
            } else {
                /* The window has not been modified, so there is nothing to do besides closing it. */
                true
            };
            self.tabview.close_page_finish(&tabpage, close_page);

            if self.tabview.selected_page().is_none() {
                /* No more tabs to present, switch to the welcome view. */
                self.stack.set_visible_child_name("welcome");
            }
        }

        pub(super) fn toast_error(&self, error: CarteroError) {
            let toast = adw::Toast::new(&error.to_string());
            self.toaster.add_toast(toast);
        }

        pub(super) fn toast_message(&self, msg: &str) {
            let toast = adw::Toast::new(msg);
            self.toaster.add_toast(toast);
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
            if cfg!(target_os = "macos") {
                let obj = self.obj();
                obj.add_css_class("macos");
            }

            self.init_settings();

            self.tabview.connect_close_page(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or]
                glib::Propagation::Stop,
                move |_, tabpage| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        tabpage,
                        async move {
                            imp.close_tab_requested(&tabpage).await;
                            imp.save_visible_tabs();
                        }
                    ));
                    glib::Propagation::Stop
                }
            ));

            self.tabview.connect_page_reordered(glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |_, _, _| {
                    window.save_visible_tabs();
                }
            ));

            let action_new = ActionEntry::builder("new")
                .activate(glib::clone!(
                    #[weak(rename_to = window)]
                    self,
                    move |_, _, _| {
                        window.action_new_endpoint();
                    }
                ))
                .build();

            let action_request = ActionEntry::builder("request")
                .activate(glib::clone!(
                    #[weak(rename_to = window)]
                    self,
                    move |_, _, _| {
                        glib::spawn_future_local(glib::clone!(
                            #[weak]
                            window,
                            async move {
                                if let Some(pane) = window.current_pane().and_then(|e| e.endpoint())
                                {
                                    if let Err(e) = pane.perform_request().await {
                                        window.toast_error(e);
                                    }
                                }
                            }
                        ));
                    }
                ))
                .build();
            let action_open = ActionEntry::builder("open")
                .activate(move |window: &super::CarteroWindow, _, _| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        window,
                        async move {
                            let imp = window.imp();
                            imp.action_open_endpoint().await;
                        }
                    ));
                })
                .build();
            let action_save = ActionEntry::builder("save")
                .activate(move |window: &super::CarteroWindow, _, _| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        window,
                        async move {
                            let imp = window.imp();
                            imp.action_save_endpoint().await;
                        }
                    ));
                })
                .build();
            let action_save_as = ActionEntry::builder("save-as")
                .activate(move |window: &super::CarteroWindow, _, _| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        window,
                        async move {
                            let imp = window.imp();
                            imp.action_save_endpoint_as().await;
                        }
                    ));
                })
                .build();
            let action_close = ActionEntry::builder("close")
                .activate(glib::clone!(
                    #[weak(rename_to = window)]
                    self,
                    move |_, _, _| {
                        if let Some(page) = window.tabview.selected_page() {
                            window.tabview.close_page(&page);
                        }
                    }
                ))
                .build();

            let action_about = ActionEntry::builder("about")
                .activate(glib::clone!(
                    #[weak(rename_to = window)]
                    self,
                    move |_, _, _| {
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
                        if cfg!(target_os = "macos") {
                            about.add_css_class("macos");
                        }
                        about.present();
                    }
                ))
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
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {
        fn close_request(&self) -> glib::Propagation {
            let has_dirty = self.iter_panes().any(|pane| pane.dirty());
            if has_dirty {
                glib::spawn_future_local(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    async move {
                        let obj = imp.obj();
                        if dialogs::confirm_close_window(&*obj).await {
                            imp.save_window_state();
                            obj.destroy();
                        }
                    }
                ));
                glib::Propagation::Stop
            } else {
                self.save_window_state();
                glib::Propagation::Proceed
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

    pub async fn open_endpoints(
        &self,
        files: &[gio::File],
    ) -> IndexMap<ItemPane, Option<FileLoadFailure>> {
        let imp = self.imp();
        imp.open_endpoints(files).await
    }

    pub async fn report_open_endpoints_errors(
        &self,
        opened: &IndexMap<ItemPane, Option<FileLoadFailure>>,
    ) {
        let imp = self.imp();
        imp.report_open_endpoints_errors(opened).await
    }

    pub fn toast_error(&self, e: CarteroError) {
        let imp = self.imp();
        imp.toast_error(e);
    }

    pub fn toast_message(&self, msg: &str) {
        let imp = self.imp();
        imp.toast_message(msg);
    }
}
