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
use gtk::{gio, glib};

mod imp {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use adw::prelude::{AdwDialogExt, ComboRowExt};
    use adw::subclass::dialog::AdwDialogImpl;
    use glib::object::{Cast, CastNone};
    use glib::subclass::InitializingObject;
    use glib::types::StaticType;
    use glib::Object;
    use gtk::gio::ListStore;
    use gtk::prelude::{EditableExt, ListItemExt, SettingsExtManual, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtk::{glib, Button, CompositeTemplate, Image, Label, ListView, SignalListItemFactory};

    use crate::app::CarteroApplication;
    use crate::entities::EndpointData;
    use crate::error::CarteroError;
    use crate::fs::collection::open_collection;
    use crate::objects::{TreeNode, TreeNodeKind};
    use crate::win::CarteroWindow;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/new_request_window.ui")]
    pub struct NewRequestWindow {
        #[template_child]
        request_name: TemplateChild<adw::EntryRow>,

        #[template_child]
        request_name_taken: TemplateChild<Image>,

        #[template_child]
        request_collection: TemplateChild<adw::ComboRow>,

        #[template_child]
        model_collection: TemplateChild<gtk::SingleSelection>,

        #[template_child]
        create_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewRequestWindow {
        const NAME: &'static str = "CarteroNewRequestWindow";
        type Type = super::NewRequestWindow;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NewRequestWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_tree_model();
            self.populate_collections();
            self.create_button.set_sensitive(false);
        }
    }

    impl WidgetImpl for NewRequestWindow {}

    impl AdwDialogImpl for NewRequestWindow {}

    #[gtk::template_callbacks]
    impl NewRequestWindow {
        fn init_tree_model(&self) {
            let model: ListStore = Object::builder()
                .property("item-type", TreeNode::static_type())
                .build();
            self.model_collection.set_model(Some(&model));
        }

        fn populate_collections(&self) {
            let app = CarteroApplication::get();
            let settings = app.settings();
            let collections: Vec<String> = settings.get("open-collections");

            let model = self.root_model().unwrap();

            for collection in collections {
                let path = PathBuf::from(&collection);

                if let Ok(node) = open_collection(&path) {
                    let treenode = TreeNode::new();
                    treenode.set_path(collection.clone());
                    treenode.set_title(node.title().replace(".cartero", ""));
                    treenode.set_node_type(TreeNodeKind::Collection);
                    model.append(&treenode);
                }
            }
        }

        pub(super) fn root_model(&self) -> Option<ListStore> {
            self.model_collection.model().and_downcast::<ListStore>()
        }

        fn create_placeholder(&self, path: &PathBuf) -> Result<(), CarteroError> {
            let ep = EndpointData::default();
            let toml = crate::file::store_toml(&ep)?;
            let mut file = File::create(path)?;
            write!(file, "{}", toml)?;
            Ok(())
        }

        fn cartero_window(&self) -> CarteroWindow {
            let obj = self.obj();
            obj.root()
                .and_downcast::<CarteroWindow>()
                .unwrap()
                .to_owned()
        }

        #[template_callback]
        fn on_create_request(&self) {
            if let Some(path) = self.requested_file() {
                if !path.exists() {
                    let _ = self.create_placeholder(&path);

                    let win = self.cartero_window();
                    let gio_file = gtk::gio::File::for_path(&path);
                    glib::spawn_future_local(glib::clone!(@weak win => async move {
                        win.add_endpoint(Some(&gio_file)).await;
                    }));
                }

                let obj = self.obj();
                obj.close();
            }
        }

        #[template_callback]
        fn on_collection_factory_setup(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            let widget: Label = Object::builder().build();
            item.set_child(Some(&widget));
        }

        #[template_callback]
        fn on_collection_factory_bind(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            let widget = item.child().and_downcast::<Label>().unwrap();
            let node = item.item().and_downcast::<TreeNode>().unwrap();
            widget.set_label(&node.pretty_name());
        }

        #[template_callback]
        fn on_collection_factory_unbind(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            let widget = item.child().and_downcast::<Label>().unwrap();
            widget.set_label("");
        }

        #[template_callback]
        fn on_collection_factory_teardown(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            item.set_child(Option::<&gtk::Widget>::None);
        }

        #[template_callback]
        fn on_request_name_changed(&self) {
            self.validate_form();
        }

        #[template_callback]
        fn on_collection_changed(&self) {
            self.validate_form();
        }

        fn requested_file(&self) -> Option<PathBuf> {
            let name = self.request_name.text();
            self.request_collection
                .selected_item()
                .and_downcast::<TreeNode>()
                .map(|collection| {
                    let path = PathBuf::from(&collection.path());
                    let file_name = format!("{}.cartero", name);
                    path.join(&file_name)
                })
        }

        fn validate_form(&self) {
            let name = self.request_name.text();
            let valid = if name.is_empty() {
                RequestWindowValidation::FileEmpty
            } else {
                match self.requested_file() {
                    Some(file) => {
                        if file.exists() {
                            RequestWindowValidation::FileTaken
                        } else {
                            RequestWindowValidation::Valid
                        }
                    }
                    None => RequestWindowValidation::FileEmpty,
                }
            };
            self.request_name_taken
                .set_visible(valid == RequestWindowValidation::FileTaken);
            self.create_button
                .set_sensitive(valid == RequestWindowValidation::Valid);
        }
    }

    #[derive(Eq, PartialEq)]
    enum RequestWindowValidation {
        Valid,
        FileEmpty,
        FileTaken,
    }
}

glib::wrapper! {
    pub struct NewRequestWindow(ObjectSubclass<imp::NewRequestWindow>)
    @extends gtk::Widget, adw::Dialog,
    @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
        gtk::ConstraintTarget, gtk::Actionable, gtk::ActionBar, gtk::ATContext;
}

impl NewRequestWindow {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
