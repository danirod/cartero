// Copyright 2024-2026 the Cartero authors
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

use cartero_objects::{Field, FieldTable};
use std::error::Error as StdError;

mod auth;
mod body;
mod environment;
mod request;
mod url;

pub use environment::*;
pub use request::BoundRequest;

#[derive(Debug)]
pub enum RequestError {
    EmptyUrl,
    UrlBadParse,
    MissingProtocol,
    UnsupportedProtocol(String), // Protocol {} not supported

    VariableNotFound(String), // Variable {} not found
    BadInterpolation,         // (variable interpolation)

    InvalidHeaderName(String),  // Header {} is invalid
    InvalidHeaderValue(String), // Header {} has an invalid value

    EncodingError, // (body encoding)

    FilePrefixUnset,      // there is no prefix, so I cannot derive paths for files.
    UnsecureFile(String), // access a file that is outside the prefix.

    // network error during the request (the type is up to the implementor)
    ProxyConfigError(String),
    NetworkError(Box<dyn StdError>),
    IOError(Box<dyn StdError>),
}

impl From<srtemplate::Error> for RequestError {
    fn from(value: srtemplate::Error) -> Self {
        match value {
            srtemplate::Error::VariableNotFound(var) => Self::VariableNotFound(var),
            _ => Self::BadInterpolation,
        }
    }
}

use gio::prelude::ListModelExtManual;

pub(crate) fn active_pairs(table: &FieldTable) -> Vec<(String, String)> {
    table
        .iter::<Field>()
        .filter_map(|elem| elem.ok())
        .filter(Field::active)
        .map(|field| (field.key(), field.value()))
        .collect::<Vec<(String, String)>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_pairs() {
        let field_table = FieldTable::from_iter(vec![
            Field::builder().key("category").value("10").build(),
            Field::builder()
                .key("user_id")
                .value("200")
                .active(false)
                .build(),
            Field::builder().key("superuser").value("true").build(),
        ]);

        let pairs = active_pairs(&field_table);
        assert_eq!(2, pairs.len());

        // The order is important at this point since there might be overrides.
        assert_eq!("category", pairs[0].0);
        assert_eq!("10", pairs[0].1);
        assert_eq!("superuser", pairs[1].0);
        assert_eq!("true", pairs[1].1);
    }
}
