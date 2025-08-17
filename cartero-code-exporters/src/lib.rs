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

mod curl;
mod plain;

pub enum Format {
    Curl,
}

fn template(format: Format, request: plain::Request) -> Box<impl askama::Template> {
    match format {
        Format::Curl => Box::new(curl::CurlTemplate { request }),
    }
}

#[derive(Debug, Clone)]
pub enum ExportError {
    VariableNotFound(String),
    BadInterpolation,
    TemplateError(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::VariableNotFound(var) => write!(f, "variable not found: {}", var),
            Self::BadInterpolation => write!(f, "bad interpolation"),
            Self::TemplateError(cause) => write!(f, "inner template error: {}", cause),
        }
    }
}

impl From<srtemplate::Error> for ExportError {
    fn from(value: srtemplate::Error) -> Self {
        match value {
            srtemplate::Error::VariableNotFound(var) => Self::VariableNotFound(var),
            _ => Self::BadInterpolation,
        }
    }
}

impl From<askama::Error> for ExportError {
    fn from(value: askama::Error) -> Self {
        ExportError::TemplateError(value.to_string())
    }
}

pub fn export_request(
    format: Format,
    request: &cartero_objects::Request,
) -> Result<String, ExportError> {
    let plain: plain::Request = request.resolve()?.into();
    let renderer = template(format, plain);
    renderer
        .render()
        .map(|s| s.trim().to_string())
        .map_err(ExportError::from)
}
