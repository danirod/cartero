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
use std::path::PathBuf;

mod imp {
    use std::cell::RefCell;

    use glib::Properties;
    use gtk::glib;
    use gtk::glib::subclass::prelude::*;
    use gtk::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TreeNode)]
    pub struct TreeNode {
        #[property(get, set)]
        path: RefCell<String>,

        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set, builder(super::TreeNodeKind::default()))]
        node_type: RefCell<super::TreeNodeKind>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TreeNode {
        const NAME: &'static str = "CarteroTreeNode";
        type Type = super::TreeNode;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TreeNode {}
}

glib::wrapper! {
    pub struct TreeNode(ObjectSubclass<imp::TreeNode>);
}

impl Default for TreeNode {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeNode {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn pretty_name(&self) -> String {
        let title = self.title();
        if title.is_empty() {
            let path = PathBuf::from(self.path());
            match path.file_name() {
                Some(name) => name.to_os_string().into_string().unwrap(),
                None => self.path(),
            }
        } else {
            title
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroTreeNodeKind")]
pub enum TreeNodeKind {
    #[default]
    Collection,
    Folder,
    Endpoint,
}
