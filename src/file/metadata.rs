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

use serde::{Deserialize, Serialize};

use crate::{
    entities::KeyValueTable,
    objects::{Collection, KeyValueItem},
};

use super::KeyValuedFileTable;

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    title: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CarteroMetadata {
    collection: Metadata,
    variables: Option<KeyValuedFileTable>,
}

impl CarteroMetadata {
    pub fn assign_properties(&self, object: &Collection) {
        object.set_title(self.collection.title.clone());
        object.set_description(self.collection.description.clone());
        object.variables().clear();
        if let Some(variables) = &self.variables {
            let kvt = KeyValueTable::from(variables.clone());
            for kv in &*kvt {
                let kvi = KeyValueItem::from(kv.clone());
                object.variables().insert(&kvi);
            }
        }
    }
}

impl From<&Collection> for CarteroMetadata {
    fn from(value: &Collection) -> Self {
        let variables = value.variables();
        Self {
            collection: Metadata {
                title: value.title(),
                description: value.description(),
            },
            variables: Some(KeyValuedFileTable::from(variables)),
        }
    }
}
