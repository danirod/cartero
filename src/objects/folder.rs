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

use std::{collections::HashMap, slice::Iter};

use super::{tree::TreeNode, Endpoint};

pub struct Folder {
    pub name: String,
    pub variables: HashMap<String, String>,
    pub nodes: Vec<TreeNode>,
}

impl Folder {
    fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            variables: HashMap::default(),
            nodes: Vec::default(),
        }
    }

    pub fn add_folder(&mut self, folder: Folder) {
        self.nodes.push(TreeNode::Folder(folder));
    }

    pub fn add_endpoint(&mut self, endpoint: Endpoint) {
        self.nodes.push(TreeNode::Endpoint(endpoint));
    }

    pub fn nodes_iter(&self) -> Iter<TreeNode> {
        self.nodes.iter()
    }
}

#[cfg(test)]
mod test {
    use crate::{client::Request, objects::Endpoint};

    use super::*;

    #[test]
    fn test_creates_a_empty_folder() {
        let folder = Folder::new("CRUD");
        assert_eq!(&folder.name, "CRUD");
        assert_eq!(folder.variables.len(), 0);
        assert_eq!(folder.nodes.len(), 0);
    }

    #[test]
    fn test_can_hold_a_endpoint() {
        let req = Request::new(
            "https://www.google.es".into(),
            crate::client::RequestMethod::Get,
            HashMap::default(),
            Vec::default(),
        );
        let ep = Endpoint(req, HashMap::default());

        let mut folder = Folder::new("CRUD");
        folder.add_endpoint(ep);

        assert_eq!(&folder.name, "CRUD");
        assert_eq!(folder.variables.len(), 0);
        assert_eq!(folder.nodes.len(), 1);

        let inner_req = match &folder.nodes[0] {
            TreeNode::Endpoint(ep) => &ep.0,
            TreeNode::Folder(_) => panic!("not an endpoint"),
        };
        assert_eq!(inner_req.url, "https://www.google.es");
    }

    #[test]
    fn test_can_hold_a_folder() {
        let node = Folder::new("CRUD");
        let mut root = Folder::new("ROOT");
        root.add_folder(node);

        assert_eq!(&root.name, "ROOT");
        assert_eq!(root.variables.len(), 0);
        assert_eq!(root.nodes.len(), 1);

        let inner_folder = match &root.nodes[0] {
            TreeNode::Endpoint(_) => panic!("not a folder"),
            TreeNode::Folder(fol) => fol,
        };
        assert_eq!(&inner_folder.name, "CRUD");
    }

    #[test]
    fn test_nodes_iter() {
        let mut node = Folder::new("CRUD");

        let subfolder = Folder::new("Subfolder");
        node.add_folder(subfolder);

        let request = Request::new(
            "https://www.gnome.org".into(),
            crate::client::RequestMethod::Get,
            HashMap::default(),
            Vec::default(),
        );
        let endpoint = Endpoint(request, HashMap::default());
        node.add_endpoint(endpoint);

        let iter = node.nodes_iter();
        assert_eq!(2, iter.len());

        let nodes = iter.as_slice();
        let _ = match &nodes[0] {
            TreeNode::Folder(_) => (),
            _ => panic!("not a folder"),
        };
        let _ = match &nodes[1] {
            TreeNode::Endpoint(_) => (),
            _ => panic!("not an endpoint"),
        };
    }
}
