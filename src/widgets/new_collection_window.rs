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

use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{gio, prelude::*};

mod imp {
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;
    use gtk::Button;
    use gtk::CompositeTemplate;
    use gtk::FileDialog;
    use gtk::Image;

    use crate::win::CarteroWindow;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/new_collection_window.ui")]
    pub struct NewCollectionWindow {
        #[template_child]
        collection_name: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub(super) collection_location: TemplateChild<adw::EntryRow>,

        #[template_child]
        file_dialog: TemplateChild<FileDialog>,

        #[template_child]
        file_taken: TemplateChild<Image>,

        #[template_child]
        parent_directory_does_not_exist: TemplateChild<Image>,

        #[template_child]
        create_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewCollectionWindow {
        const NAME: &'static str = "CarteroNewCollectionWindow";
        type Type = super::NewCollectionWindow;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NewCollectionWindow {}

    impl WidgetImpl for NewCollectionWindow {}

    impl AdwDialogImpl for NewCollectionWindow {}

    #[gtk::template_callbacks]
    impl NewCollectionWindow {
        fn cartero_window(&self) -> CarteroWindow {
            let obj = self.obj();
            obj.root()
                .and_downcast::<CarteroWindow>()
                .unwrap()
                .to_owned()
        }

        #[template_callback]
        async fn on_location_clicked(&self) {
            // Start the selection with the current directory.
            let current_dir_path = self.get_picked_directory();
            self.file_dialog.set_initial_folder(Some(&current_dir_path));

            // Ask the user for the new directory.
            let win = self.cartero_window();
            let result = self.file_dialog.select_folder_future(Some(&win)).await;

            // Update the location text field with the new value.
            if let Ok(folder) = result {
                let path = folder.path().unwrap();
                self.collection_location
                    .set_text(path.as_os_str().to_str().unwrap());
            }
        }

        // This function shall return the value of the directory entry.
        fn get_picked_directory(&self) -> gtk::gio::File {
            let current_dir = self.collection_location.text().to_string();
            gtk::gio::File::for_path(current_dir)
        }

        // Callback is called whenever the value of the collection_name textfield changes.
        #[template_callback]
        async fn on_collection_name_changed(&self) {
            self.update_dialog_validness();
        }

        #[template_callback]
        async fn on_collection_location_changed(&self) {
            self.update_dialog_validness();
        }

        // Tests whether the directory and collection are valid and updates
        // the validation of the form. May paint icons, make the create
        // button enabled or not, or paint some entries red.
        fn update_dialog_validness(&self) {
            // First, reset the
            // Get all the paths.
            let target_directory = self.get_picked_directory().path().unwrap();

            // First, let's make sure that the directory exists.
            if !target_directory.exists() {
                self.parent_directory_does_not_exist.set_visible(true);
                self.collection_location.add_css_class("error");
                self.collection_name.add_css_class("error");
                self.create_button.set_sensitive(false);
                return;
            } else {
                self.collection_location.remove_css_class("error");
                self.collection_name.remove_css_class("error");
                self.parent_directory_does_not_exist.set_visible(false);
            }

            // Make sure that the directory name does not exist.
            let full_directory = target_directory.join(self.collection_name.text());
            if !self.collection_name.text().is_empty() && full_directory.exists() {
                self.file_taken.set_visible(true);
                self.collection_name.add_css_class("error");
                self.create_button.set_sensitive(false);
                return;
            } else {
                self.file_taken.set_visible(false);
            }

            // The collection name must have been set.
            let sensitive = !self.collection_name.text().is_empty();
            self.create_button.set_sensitive(sensitive);
        }

        #[template_callback]
        fn on_create_collection(&self) {
            let directory = self.get_picked_directory().path().unwrap();
            let name = self.collection_name.text();
            let full_directory = directory.join(name);
            let outcome = self
                .cartero_window()
                .finish_create_collection(&full_directory);
            if let Err(o) = outcome {
                // TODO: Present a warning message with this.
                println!("{}", o);
            }

            let obj = self.obj();
            obj.close();
        }

        #[template_callback]
        fn on_cancel_creation(&self) {
            self.dispose();
        }
    }
}

glib::wrapper! {
    pub struct NewCollectionWindow(ObjectSubclass<imp::NewCollectionWindow>)
    @extends gtk::Widget, adw::Dialog,
    @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                gtk::ConstraintTarget, gtk::Actionable, gtk::ActionBar, gtk::ATContext;
}

impl NewCollectionWindow {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_initial_directory(&self, dir: &str) {
        let imp = self.imp();
        imp.collection_location.set_text(dir);
    }
}

impl Default for NewCollectionWindow {
    fn default() -> Self {
        Object::builder().build()
    }
}
