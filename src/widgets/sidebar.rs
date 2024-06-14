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

use glib::{subclass::types::ObjectSubclassIsExt, Object};
use gtk::{gio::Settings, prelude::SettingsExtManual};

use crate::{
    fs::collection::open_collection,
    objects::{TreeNode, TreeNodeKind},
};

mod imp {
    use adw::subclass::bin::BinImpl;
    use glib::subclass::InitializingObject;
    use glib::Object;
    use gtk::gio::{ListModel, ListStore};
    use gtk::subclass::prelude::*;
    use gtk::{
        prelude::*, CompositeTemplate, ListView, SignalListItemFactory, SingleSelection,
        TreeExpander, TreeListModel,
    };

    use crate::objects::{KeyValueItem, TreeNode, TreeNodeKind};
    use crate::widgets::sidebar_row::SidebarRow;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/sidebar.ui")]
    pub struct Sidebar {
        #[template_child]
        pub(super) selection_model: TemplateChild<SingleSelection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "CarteroSidebar";
        type Type = super::Sidebar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Sidebar {
        fn constructed(&self) {
            self.parent_constructed();

            let tree_model = self.init_tree_model();
            self.selection_model.set_model(Some(&tree_model));
        }
    }

    impl WidgetImpl for Sidebar {}

    impl BinImpl for Sidebar {}

    #[gtk::template_callbacks]
    impl Sidebar {
        fn init_tree_model(&self) -> TreeListModel {
            let root_model: ListStore = Object::builder()
                .property("item-type", TreeNode::static_type())
                .build();
            TreeListModel::new(root_model, false, false, |_obj: &Object| {
                let children: ListStore = Object::builder()
                    .property("item-type", KeyValueItem::static_type())
                    .build();
                let model = children.upcast::<ListModel>();
                Some(model)
            })
        }

        pub(super) fn root_model(&self) -> Option<ListStore> {
            self.selection_model
                .model()
                .and_downcast::<TreeListModel>()
                .map(|tlm: TreeListModel| tlm.model())
                .and_downcast::<ListStore>()
        }

        #[template_callback]
        fn on_activate(list: ListView, pos: u32, data: &Object) {
            println!("activate()");
            println!("list = {:?} \n pos = {:?} \n data = {:?}", list, pos, data);
        }

        #[template_callback]
        fn on_factory_setup(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            let row = SidebarRow::new();
            row.deactivate_actions();

            let expander = TreeExpander::new();
            expander.set_child(Some(&row));
            item.set_child(Some(&expander));
        }

        #[template_callback]
        fn on_factory_bind(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            let expander = item.child().and_downcast::<TreeExpander>().unwrap();
            let widget = expander.child().and_downcast::<SidebarRow>().unwrap();
            let row = item.item().and_downcast::<gtk::TreeListRow>().unwrap();

            expander.set_list_row(Some(&row));

            let item = row.item().and_downcast::<TreeNode>().unwrap();
            widget.set_title(item.pretty_name());
            widget.set_path(item.path());
            match item.node_type() {
                TreeNodeKind::Collection => widget.activate_collection_actions(),
                _ => widget.deactivate_actions(),
            }
        }

        #[template_callback]
        fn on_factory_unbind(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            let expander = item.child().and_downcast::<TreeExpander>().unwrap();
            let widget = expander.child().and_downcast::<SidebarRow>().unwrap();
            expander.set_list_row(None);
            widget.set_title("");
            widget.set_path("");
            widget.deactivate_actions();
        }

        #[template_callback]
        fn on_factory_teardown(_: SignalListItemFactory, obj: &Object) {
            let item = obj.downcast_ref::<gtk::ListItem>().unwrap();
            item.set_child(Option::<&gtk::Widget>::None);
        }
    }
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Sidebar {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl Sidebar {
    pub fn sync_collections(&self, settings: &Settings) {
        let collections: Vec<String> = settings.get("open-collections");
        let imp = self.imp();
        let store = imp.root_model().unwrap();

        // First, delete the list model currently in use.
        store.remove_all();

        // Then, add the collections again.
        for collection in collections {
            let collection_path = PathBuf::from(&collection);

            // TODO: Ignore this collection if the path does not exist.
            if let Ok(collection_obj) = open_collection(&collection_path) {
                let tree_node = TreeNode::new();
                tree_node.set_path(collection.clone());
                tree_node.set_title(collection_obj.title());
                tree_node.set_node_type(TreeNodeKind::Collection);
                store.append(&tree_node);
            }
        }
    }
}
