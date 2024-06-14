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

use std::collections::HashMap;

use glib::Object;
use serde::{Deserialize, Serialize};

use crate::objects::{Collection, KeyValueItem};

#[derive(Serialize, Deserialize)]
struct Information {
    title: String,
    description: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct Variable {
    active: bool,
    value: String,
    secret: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    info: Information,
    variables: Option<HashMap<String, Variable>>,
}

impl From<Metadata> for Collection {
    fn from(val: Metadata) -> Self {
        let col: Collection = Object::builder()
            .property("title", val.info.title)
            .property("description", val.info.description)
            .property("version", val.info.version)
            .build();

        if let Some(variables) = val.variables {
            for (variable_name, variable_info) in variables {
                let kv: KeyValueItem = Object::builder()
                    .property("header-name", variable_name)
                    .property("header-value", variable_info.value)
                    .property("active", variable_info.active)
                    .property("secret", variable_info.secret)
                    .build();
                col.add_variable(&kv);
            }
        }

        col
    }
}

impl From<&Collection> for Metadata {
    fn from(val: &Collection) -> Self {
        let title = val.title();
        let description = val.description();
        let version = val.version();

        let variables = val
            .variables_list()
            .iter()
            .map(|var| {
                let variable_name = var.header_name();
                let variable = Variable {
                    active: var.active(),
                    value: var.header_value(),
                    secret: var.secret(),
                };
                (variable_name.to_string(), variable)
            })
            .collect();

        Metadata {
            info: Information {
                title,
                description,
                version,
            },
            variables: Some(variables),
        }
    }
}
