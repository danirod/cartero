// Copyright 2024-2025 the Cartero authors
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

use srtemplate::SrTemplate;

use super::{KeyValueTable, RequestAuthorization, RequestMethod, RequestPayload};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EndpointData {
    pub url: String,
    pub method: RequestMethod,
    pub parameters: KeyValueTable,
    pub headers: KeyValueTable,
    pub variables: KeyValueTable,
    pub body: RequestPayload,
    pub authorization: RequestAuthorization,
}

impl EndpointData {
    pub fn template_processor(&self) -> SrTemplate {
        let context = SrTemplate::default();
        for item in self.variables.iter() {
            if item.active {
                context.add_variable(item.name.clone(), &item.value);
            }
        }
        context
    }

    pub fn process_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for item in self.headers.iter() {
            if item.active {
                headers.insert(item.name.clone(), item.value.clone());
            }
        }
        headers
    }
}
