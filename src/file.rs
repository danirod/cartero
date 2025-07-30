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

use cartero_interop::FileWarningTag;
use formatx::formatx;
use gettextrs::gettext;
use gtk::gio;
use gtk::prelude::{FileExtManual, SettingsExtManual};
use serde::{Deserialize, Serialize};

use crate::app::CarteroApplication;
use crate::entities::{
    EndpointData, KeyValue, KeyValueTable, RawEncoding, RequestAuthorization, RequestMethod,
    RequestPayload,
};
use crate::error::FileSaveError;

trait ToKeyValue {
    fn to_key_value(&self, key: &str) -> KeyValue;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum FieldValue {
    Simple(String),
    Complex {
        value: String,
        active: bool,
        secret: bool,
    },
}

impl ToKeyValue for FieldValue {
    fn to_key_value(&self, key: &str) -> KeyValue {
        match self {
            FieldValue::Simple(str) => KeyValue {
                name: key.to_owned(),
                value: str.clone(),
                active: true,
                secret: false,
            },
            FieldValue::Complex {
                value,
                active,
                secret,
            } => KeyValue {
                name: key.to_owned(),
                value: value.clone(),
                active: *active,
                secret: *secret,
            },
        }
    }
}

impl Default for FieldValue {
    fn default() -> Self {
        Self::Simple(String::default())
    }
}

impl From<KeyValue> for FieldValue {
    fn from(value: KeyValue) -> Self {
        if value.active && !value.secret {
            Self::Simple(value.value)
        } else {
            Self::Complex {
                active: value.active,
                secret: value.secret,
                value: value.value,
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum QueryValue {
    Simple(String),
    Complex { value: String, secret: bool },
}

impl ToKeyValue for QueryValue {
    fn to_key_value(&self, key: &str) -> KeyValue {
        match self {
            QueryValue::Simple(str) => KeyValue {
                name: key.to_owned(),
                value: str.clone(),
                active: false,
                secret: false,
            },
            QueryValue::Complex { value, secret } => KeyValue {
                name: key.to_owned(),
                value: value.clone(),
                active: false,
                secret: *secret,
            },
        }
    }
}

impl Default for QueryValue {
    fn default() -> Self {
        Self::Simple(String::default())
    }
}

impl From<KeyValue> for QueryValue {
    fn from(value: KeyValue) -> Self {
        if !value.secret {
            Self::Simple(value.value)
        } else {
            Self::Complex {
                secret: value.secret,
                value: value.value,
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum FileContainer<T>
where
    T: From<KeyValue> + ToKeyValue,
{
    Unique(T),
    Multiple(Vec<T>),
}

impl<T> From<Vec<KeyValue>> for FileContainer<T>
where
    T: From<KeyValue> + ToKeyValue,
{
    fn from(value: Vec<KeyValue>) -> Self {
        if value.len() == 1 {
            Self::Unique(value[0].clone().into())
        } else {
            let multiple: Vec<T> = value.into_iter().map(T::from).collect();
            Self::Multiple(multiple)
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
struct FileTable<T>(HashMap<String, FileContainer<T>>)
where
    T: From<KeyValue> + ToKeyValue;

impl<T> From<FileTable<T>> for KeyValueTable
where
    T: From<KeyValue> + ToKeyValue,
{
    fn from(value: FileTable<T>) -> Self {
        let mut vector: Vec<KeyValue> = value
            .0
            .into_iter()
            .flat_map(|(header, values)| match values {
                FileContainer::Unique(x) => vec![x.to_key_value(&header)],
                FileContainer::Multiple(mult) => {
                    mult.into_iter().map(|v| v.to_key_value(&header)).collect()
                }
            })
            .collect();
        vector.sort();
        KeyValueTable::new(&vector)
    }
}

impl<T> From<KeyValueTable> for FileTable<T>
where
    T: From<KeyValue> + ToKeyValue,
{
    fn from(value: KeyValueTable) -> Self {
        let group = value.group_by();
        let inner = group
            .into_iter()
            .map(|(key, vector)| (key, vector.into()))
            .collect();
        Self(inner)
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum FilePayloadRawFormat {
    #[default]
    #[serde(rename = "octet-stream")]
    OctetStream,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "xml")]
    Xml,
}

impl From<RawEncoding> for FilePayloadRawFormat {
    fn from(value: RawEncoding) -> Self {
        match value {
            RawEncoding::Json => Self::Json,
            RawEncoding::OctetStream => Self::OctetStream,
            RawEncoding::Xml => Self::Xml,
        }
    }
}

impl From<FilePayloadRawFormat> for RawEncoding {
    fn from(value: FilePayloadRawFormat) -> Self {
        match value {
            FilePayloadRawFormat::Json => Self::Json,
            FilePayloadRawFormat::OctetStream => Self::OctetStream,
            FilePayloadRawFormat::Xml => Self::Xml,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum FilePayload {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "urlencoded")]
    UrlEncoded {
        variables: Option<FileTable<FieldValue>>,
    },
    #[serde(rename = "multipart")]
    Multipart {
        variables: Option<FileTable<FieldValue>>,
    },
    #[serde(rename = "raw")]
    Raw {
        format: Option<FilePayloadRawFormat>,
        body: String,
    },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum FileBody {
    Raw(String),
    Structured(FilePayload),
}

impl From<RequestPayload> for FilePayload {
    fn from(value: RequestPayload) -> Self {
        match value {
            RequestPayload::None => Self::None,
            RequestPayload::Urlencoded(payload) => Self::UrlEncoded {
                variables: Some(payload.into()),
            },
            RequestPayload::Multipart { params } => Self::Multipart {
                variables: Some(params.into()),
            },
            RequestPayload::Raw { encoding, content } => Self::Raw {
                format: Some(encoding.into()),
                body: String::from_utf8_lossy(&content.clone()).to_string(),
            },
        }
    }
}

impl From<FilePayload> for RequestPayload {
    fn from(value: FilePayload) -> Self {
        match value {
            FilePayload::None => Self::None,
            FilePayload::Multipart { variables } => Self::Multipart {
                params: variables.unwrap_or_default().into(),
            },
            FilePayload::UrlEncoded { variables } => {
                Self::Urlencoded(variables.unwrap_or_default().into())
            }
            FilePayload::Raw { format, body } => Self::Raw {
                encoding: format.unwrap_or_default().into(),
                content: Vec::from(body.clone()),
            },
        }
    }
}

impl From<RequestPayload> for FileBody {
    fn from(value: RequestPayload) -> Self {
        Self::Structured(value.into())
    }
}

impl From<FileBody> for RequestPayload {
    fn from(value: FileBody) -> Self {
        match value {
            FileBody::Raw(payload) => Self::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(payload.clone().as_str()),
            },
            FileBody::Structured(payload) => payload.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "type")]
enum FileAuthorization {
    #[serde(rename = "none")]
    #[default]
    None,

    #[serde(rename = "basic")]
    Basic { username: String, password: String },

    #[serde(rename = "bearer")]
    Bearer { token: String },
}

impl From<RequestAuthorization> for FileAuthorization {
    fn from(value: RequestAuthorization) -> Self {
        match value {
            RequestAuthorization::None => Self::None,
            RequestAuthorization::Basic { username, password } => {
                Self::Basic { username, password }
            }
            RequestAuthorization::Bearer(token) => Self::Bearer { token },
        }
    }
}

impl From<FileAuthorization> for RequestAuthorization {
    fn from(value: FileAuthorization) -> Self {
        match value {
            FileAuthorization::None => Self::None,
            FileAuthorization::Basic { username, password } => Self::Basic { username, password },
            FileAuthorization::Bearer { token } => Self::Bearer(token),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct RequestFile {
    version: usize,
    url: String,
    method: String,
    body: Option<FileBody>,
    headers: Option<FileTable<FieldValue>>,
    variables: Option<FileTable<FieldValue>>,
    authorization: Option<FileAuthorization>,
    #[serde(rename = "inactive-params")]
    inactive_params: Option<FileTable<QueryValue>>,
}

impl From<EndpointData> for RequestFile {
    fn from(value: EndpointData) -> RequestFile {
        let method: &str = value.method.into();
        let body = match value.body {
            RequestPayload::None => None,
            otherwise => Some(otherwise.into()),
        };
        let headers = value.headers.into();
        let variables = value.variables.into();

        let authorization = value.authorization.into();
        let authorization = match authorization {
            FileAuthorization::None => None,
            other => Some(other),
        };

        let inactive_params: Vec<_> = value
            .parameters
            .iter()
            .filter(|v| !v.active)
            .map(Clone::clone)
            .collect();
        let inactive_params = KeyValueTable::new(&inactive_params);

        RequestFile {
            version: 1,
            url: value.url.clone(),
            method: method.to_owned(),
            body,
            headers: Some(headers),
            variables: Some(variables),
            inactive_params: Some(inactive_params.into()),
            authorization,
        }
    }
}

pub fn pretty_warning(warning: FileWarningTag) -> String {
    match warning {
            FileWarningTag::InvalidHttpVerb(v) => formatx!(
                gettext("The HTTP verb found in the file was '{}'. It is not valid, it will fallback to '{}'."),
                v, "GET").unwrap()
        }
}

/// Write the contents of the given endpoint into the given file. The result
/// only notifies about errors during save, such as invalid permissions or
/// stuff like that. Note that unlike the read_endpoint() function, this is a
/// Result, because there are no partial saves.
pub async fn write_endpoint(
    file: &gio::File,
    endpoint: &EndpointData,
) -> Result<(), FileSaveError> {
    /* Serialize into a TOML document. */
    let encoded_file = RequestFile::from(endpoint.clone());
    let encoded_string = toml::to_string(&encoded_file)
        .map_err(|ser_err| FileSaveError::SerializationError(ser_err))?;

    /* Delegate saving into GIO. */
    let use_backups = create_file_backup();
    file.replace_contents_future(
        encoded_string,
        None,
        use_backups,
        gio::FileCreateFlags::NONE,
    )
    .await
    .map_err(|(_, glib_error)| FileSaveError::FileWriteError(glib_error))?;

    Ok(())
}

#[cfg(test)]
fn write_endpoint_string(endpoint: &EndpointData) -> Result<String, FileSaveError> {
    let encoded_file = RequestFile::from(endpoint.clone());
    toml::to_string(&encoded_file).map_err(|ser_err| FileSaveError::SerializationError(ser_err))
}

/// Checks the settings to guess whether backups have to be created on save.
fn create_file_backup() -> bool {
    let app = CarteroApplication::default();
    let settings = app.settings();
    settings.get::<bool>("create-backup-files")
}
