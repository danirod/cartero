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

use askama::Template;
use base64::prelude::*;

use crate::plain::*;

#[derive(Template)]
#[template(path = "../templates/curl.j2", escape = "none")]
pub struct CurlTemplate {
    pub(crate) request: Request,
}

impl CurlTemplate {
    fn unquote(&self, string: &str) -> String {
        string.replace("'", "'\"'\"'")
    }
}

#[derive(Template)]
#[template(path = "../templates/ijhttp.j2", escape = "none")]
pub struct IjhttpTemplate {
    pub(crate) request: Request,
}

impl IjhttpTemplate {
    fn get_header(&self, name: &str) -> Option<String> {
        self.request
            .headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }

    fn boundary(&self) -> String {
        let input = format!("{} {}", self.request.method, self.request.url);
        BASE64_STANDARD.encode(&input)
    }
}
