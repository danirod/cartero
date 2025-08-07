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

#![doc = include_str!("../README.md")]

mod field;
mod field_table;
mod request;
mod request_authentication;
mod request_authentication_basic;
mod request_authentication_bearer;
mod request_authentication_data;
mod request_body;
mod request_body_data;
mod request_body_file;
mod request_body_multipart;
mod request_body_raw;
mod request_body_urlencoded;
mod request_method;
mod response;
mod utils;

pub use crate::field::*;
pub use crate::field_table::*;
pub use crate::request::*;
pub use crate::request_authentication::*;
pub use crate::request_authentication_basic::*;
pub use crate::request_authentication_bearer::*;
pub use crate::request_authentication_data::*;
pub use crate::request_body::*;
pub use crate::request_body_data::*;
pub use crate::request_body_file::*;
pub use crate::request_body_multipart::*;
pub use crate::request_body_raw::*;
pub use crate::request_body_urlencoded::*;
pub use crate::request_method::*;
pub use crate::response::*;

#[cfg(test)]
#[ctor::ctor]
fn register_types() {
    use glib::types::StaticType;

    FieldTable::static_type();
}
