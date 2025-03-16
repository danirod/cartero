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

use super::KeyValueTable;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum RawEncoding {
    Json,
    Xml,
    #[default]
    OctetStream,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum RequestPayload {
    #[default]
    None,
    Urlencoded(KeyValueTable),
    Multipart {
        params: KeyValueTable,
    },
    Raw {
        encoding: RawEncoding,
        content: Vec<u8>,
    },
}
