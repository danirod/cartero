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

use base64::prelude::*;

use crate::{
    entities::{EndpointData, RequestAuthorization},
    error::RequestPreconditionError,
};

pub(super) struct BoundAuthorization {
    pub headers: Vec<(String, String)>,
}

impl TryFrom<&EndpointData> for BoundAuthorization {
    type Error = RequestPreconditionError;

    fn try_from(value: &EndpointData) -> Result<Self, Self::Error> {
        let processor = value.template_processor();

        let authorization = match &value.authorization {
            RequestAuthorization::None => None,
            RequestAuthorization::Basic { username, password } => {
                let username = processor.render(&username)?;
                let password = processor.render(&password)?;
                let hash = BASE64_STANDARD.encode(format!("{username}:{password}"));
                let encoded = format!("Basic {hash}");
                Some(encoded.to_string())
            }
            RequestAuthorization::Bearer(token) => {
                let token = processor.render(&token)?;
                let encoded = format!("Bearer {token}");
                Some(encoded.to_string())
            }
        };

        let headers = match authorization {
            None => vec![],
            Some(value) => vec![("Authorization".to_string(), value)],
        };

        Ok(Self { headers })
    }
}
