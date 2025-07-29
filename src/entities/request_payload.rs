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

impl Into<cartero_objects::RequestBody> for RequestPayload {
    fn into(self) -> cartero_objects::RequestBody {
        match self {
            RequestPayload::None => cartero_objects::RequestBody::builder().none().build(),
            RequestPayload::Urlencoded(table) => {
                let field_table = table.into();
                let urlencoded = cartero_objects::RequestBodyUrlencoded::builder()
                    .params(&field_table)
                    .build();
                cartero_objects::RequestBody::builder()
                    .urlencoded(&urlencoded)
                    .build()
            }
            RequestPayload::Multipart { params } => {
                let field_table = params.into();
                let multipart = cartero_objects::RequestBodyMultipart::builder()
                    .params(&field_table)
                    .build();
                cartero_objects::RequestBody::builder()
                    .multipart(&multipart)
                    .build()
            }
            RequestPayload::Raw { encoding, content } => {
                let encoding = match encoding {
                    RawEncoding::Json => cartero_objects::RequestBodyRawType::Json,
                    RawEncoding::Xml => cartero_objects::RequestBodyRawType::Xml,
                    RawEncoding::OctetStream => cartero_objects::RequestBodyRawType::OctetStream,
                };
                let body = cartero_objects::RequestBodyRaw::builder(encoding)
                    .payload(String::from_utf8_lossy(&content))
                    .build();
                cartero_objects::RequestBody::builder().raw(&body).build()
            }
        }
    }
}

impl From<cartero_objects::RequestBody> for RequestPayload {
    fn from(value: cartero_objects::RequestBody) -> Self {
        match value.body_type() {
            cartero_objects::RequestBodyType::None => Self::None,
            cartero_objects::RequestBodyType::UrlEncoded => {
                let urlencoded = value.urlencoded().unwrap();
                Self::Urlencoded(urlencoded.params().into())
            }
            cartero_objects::RequestBodyType::Multipart => {
                let multipart = value.multipart().unwrap();
                Self::Multipart {
                    params: multipart.params().into(),
                }
            }
            cartero_objects::RequestBodyType::Raw => {
                let raw = value.raw().unwrap();
                let encoding = match raw.payload_type() {
                    cartero_objects::RequestBodyRawType::Json => RawEncoding::Json,
                    cartero_objects::RequestBodyRawType::Xml => RawEncoding::Xml,
                    cartero_objects::RequestBodyRawType::OctetStream => RawEncoding::OctetStream,
                };
                Self::Raw {
                    encoding,
                    content: raw.payload().into(),
                }
            }
        }
    }
}
