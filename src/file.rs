use std::collections::HashMap;

use gtk::gio;
use gtk::prelude::{FileExtManual, SettingsExtManual};
use serde::{Deserialize, Serialize};

use crate::app::CarteroApplication;
use crate::entities::{
    EndpointData, KeyValue, KeyValueTable, RawEncoding, RequestMethod, RequestPayload,
};
use crate::error::{FileLoadError, FileSaveError};
use crate::i18n::i18n_f;

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

#[derive(Clone, Deserialize, Serialize)]
struct RequestFile {
    version: usize,
    url: String,
    method: String,
    body: Option<FileBody>,
    headers: Option<FileTable<FieldValue>>,
    variables: Option<FileTable<FieldValue>>,
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
        }
    }
}

/// A warning is a recoverable error, like a linter error. Something that
/// indicates that the file will not be read correctly, but the user interface
/// can still process the file. Warnings should be presented to the user to
/// let them know what's wrong with the loaded data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileWarningTag {
    /// The file had an invalid HTTP verb, and it has been reset to the default.
    InvalidHttpVerb(String),
}

impl std::fmt::Display for FileWarningTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let localized = match self {
            FileWarningTag::InvalidHttpVerb(v) => i18n_f(
                "The HTTP verb found in the file was '{}'. It is not valid, it will fallback to '{}'.",
                &[&v, "GET"],
            ),
        };
        write!(f, "{}", localized)
    }
}

pub trait FileLoadResult {
    /// Generates a generic failure result of the implementor type for anonymous panes.
    fn anonymous() -> Self;

    /// Returns possibly a failure result, either unrecoverable errors, or recoverable errors.
    fn failure(&self) -> Option<FileLoadFailure>;

    /// Returns true if the file load result concluded at something. Note that this is not
    /// the same as checking if failure() returns None, because a result may conclude and
    /// load something while at the same time having recoverable failures.
    fn loaded(&self) -> bool;

    fn result(&self) -> Result<(), FileLoadFailure> {
        match self.failure() {
            None => Ok(()),
            Some(fail) => Err(fail),
        }
    }
}

/// The result of attempting to load a file. This is not a binary operation,
/// because under some circumstances, the result may complete with warnings,
/// and it's important to notice about them.
///
/// In the past I tried to just use a Result<T, E> where T is a composite
/// structure that holds both the data structure and a maybe-empty list of
/// warnings, but this doesn't scale and creates akward code requiring to
/// pass a lot of Results of tuples.
///
/// Note that there is no EndpointSaveResult because it is assumed that there
/// are no warnings during save. If the request could not be saved properly,
/// an error would already have been triggered by the time the save is called.
#[derive(Debug, Eq, PartialEq)]
pub enum EndpointLoadResult {
    /// The file was read correctly with no errors, and here is the file.
    Success(EndpointData),

    /// The file was read with a few errors, here is what we got and the list
    /// of errors that were found while trying to read the file.
    Warning(EndpointData, Vec<FileWarningTag>),

    /// The file cannot be loaded due to a critical error that needs review.
    Error(FileLoadError),
}

impl EndpointLoadResult {
    pub fn endpoint(&self) -> Option<EndpointData> {
        match self {
            Self::Success(endpoint) => Some(endpoint.clone()),
            Self::Warning(endpoint, _) => Some(endpoint.clone()),
            Self::Error(_) => None,
        }
    }
}

impl FileLoadResult for EndpointLoadResult {
    fn failure(&self) -> Option<FileLoadFailure> {
        match self {
            Self::Success(_) => None,
            Self::Warning(_, warnings) => Some(FileLoadFailure {
                warnings: warnings.clone(),
                ..Default::default()
            }),
            Self::Error(e) => Some(FileLoadFailure {
                error: Some(e.clone()),
                ..Default::default()
            }),
        }
    }

    fn loaded(&self) -> bool {
        match self {
            Self::Error(_) => false,
            _ => true,
        }
    }

    fn anonymous() -> Self {
        Self::Error(FileLoadError::AnonymousPane)
    }
}

#[derive(Clone, Default)]
pub struct FileLoadFailure {
    pub warnings: Vec<FileWarningTag>,
    pub error: Option<FileLoadError>,
}

impl From<RequestFile> for EndpointLoadResult {
    fn from(value: RequestFile) -> Self {
        /* Make sure that the application is updated. */
        if value.version < 1 || value.version > 1 {
            return Self::Error(FileLoadError::OutdatedSchema);
        }

        /* HTTP verb is currently the only thing that could be invalid. */
        let (method, method_tag) = match RequestMethod::try_from(value.method.as_str()) {
            Ok(method) => (method, None),
            Err(_) => (
                RequestMethod::default(),
                Some(FileWarningTag::InvalidHttpVerb(value.method.clone())),
            ),
        };

        /* Every other data currently doesn't emit warnings. */
        let body = value.body.map(RequestPayload::from).unwrap_or_default();
        let headers = value.headers.unwrap_or_default().into();
        let variables = value.variables.unwrap_or_default().into();
        let inactive_params = value.inactive_params.unwrap_or_default().into();

        /* Therefore we can craft the response. */
        let request = EndpointData {
            url: value.url.clone(),
            method,
            body,
            variables,
            headers,
            parameters: inactive_params,
        };

        /* The result depends on whether there are tags. */
        match method_tag {
            Some(warning) => Self::Warning(request, vec![warning]),
            None => Self::Success(request),
        }
    }
}

/// Read the contents of the given file as an endpoint. The result that this
/// function returns is custom because it has three states: success, error,
/// or partial failure, with recoverable errors.
pub async fn read_endpoint(file: &gio::File) -> EndpointLoadResult {
    let file_string = match file.load_contents_future().await {
        Ok((data, _)) => String::from_utf8_lossy(&data).to_string(),
        Err(glib_error) => {
            return EndpointLoadResult::Error(FileLoadError::FileReadError(glib_error))
        }
    };
    match toml::from_str::<RequestFile>(&file_string) {
        Ok(contents) => EndpointLoadResult::from(contents),
        Err(e) => EndpointLoadResult::Error(FileLoadError::DeserializationError(e)),
    }
}

#[cfg(test)]
fn read_endpoint_string(contents: &str) -> EndpointLoadResult {
    match toml::from_str::<RequestFile>(&contents) {
        Ok(contents) => EndpointLoadResult::from(contents),
        Err(e) => EndpointLoadResult::Error(FileLoadError::DeserializationError(e)),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        entities::{
            EndpointData, KeyValue, KeyValueTable, RawEncoding, RequestMethod, RequestPayload,
        },
        error::FileLoadError,
        file::{EndpointLoadResult, FileWarningTag},
    };

    use super::{FileContainer, FileTable};

    #[test]
    pub fn test_key_valued_file_table_to_key_value_table_sorts_simple() {
        let map = HashMap::from([
            (
                "User-Agent".into(),
                FileContainer::Unique(super::FieldValue::Simple("Cartero/0.1".into())),
            ),
            (
                "Host".into(),
                FileContainer::Unique(super::FieldValue::Simple("www.google.com".into())),
            ),
        ]);
        let file = FileTable(map);

        let table = KeyValueTable::from(file);

        assert_eq!(table.len(), 2);
        assert_eq!(
            table,
            KeyValueTable::new(&[
                ("Host", "www.google.com").into(),
                ("User-Agent", "Cartero/0.1").into()
            ])
        );
    }

    #[test]
    pub fn test_key_valued_file_table_to_key_value_table_sorts_complex() {
        let map = HashMap::from([
            (
                "User-Agent".into(),
                FileContainer::Unique(super::FieldValue::Complex {
                    value: "Cartero/0.1".into(),
                    active: false,
                    secret: true,
                }),
            ),
            (
                "Host".into(),
                FileContainer::Unique(super::FieldValue::Simple("www.google.com".into())),
            ),
        ]);
        let file = FileTable(map);

        let table = KeyValueTable::from(file);

        assert_eq!(table.len(), 2);
        assert_eq!(
            table,
            KeyValueTable::new(&[
                ("Host", "www.google.com").into(),
                KeyValue {
                    name: "User-Agent".into(),
                    value: "Cartero/0.1".into(),
                    active: false,
                    secret: true,
                }
            ])
        );
    }

    #[test]
    pub fn test_key_valued_file_table_to_key_value_table_sorts_multiple() {
        let map = HashMap::from([
            (
                "User-Agent".into(),
                FileContainer::Unique(super::FieldValue::Simple("Cartero/0.1".into())),
            ),
            (
                "Accept".into(),
                FileContainer::Multiple(vec![
                    super::FieldValue::Simple("*/*".into()),
                    super::FieldValue::Simple("application/json".into()),
                    super::FieldValue::Simple("application/ld+json".into()),
                ]),
            ),
            (
                "Host".into(),
                FileContainer::Unique(super::FieldValue::Simple("www.google.com".into())),
            ),
        ]);
        let file = FileTable(map);

        let table = KeyValueTable::from(file);

        assert_eq!(table.len(), 5);
        assert_eq!(
            table,
            KeyValueTable::new(&vec![
                ("Accept", "*/*").into(),
                ("Accept", "application/json").into(),
                ("Accept", "application/ld+json").into(),
                ("Host", "www.google.com").into(),
                ("User-Agent", "Cartero/0.1").into(),
            ])
        );
    }

    #[test]
    pub fn test_can_deserialize_classic() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = 'text/html'
Accept-Encoding = 'gzip'
";
        let EndpointLoadResult::Success(endpoint) = super::read_endpoint_string(toml) else {
            panic!("wrong read");
        };
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 2);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&[
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: true,
                    secret: false
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                }
            ]),
        );
    }

    #[test]
    pub fn test_can_deserialize_complex_headers() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = { value = 'text/html', secret = true, active = false }
Accept-Encoding = 'gzip'
";
        let EndpointLoadResult::Success(endpoint) = super::read_endpoint_string(toml) else {
            panic!("wrong read");
        };
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 2);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&[
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: false,
                    secret: true,
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                }
            ]),
        );
    }

    #[test]
    pub fn test_can_deserialize_header_arrays() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = ['application/json', 'text/html']
Accept-Encoding = 'gzip'
";
        let EndpointLoadResult::Success(endpoint) = super::read_endpoint_string(toml) else {
            panic!("wrong read");
        };
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 3);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&[
                KeyValue {
                    name: "Accept".into(),
                    value: "application/json".into(),
                    active: true,
                    secret: false,
                },
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: true,
                    secret: false,
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                }
            ]),
        );
    }

    #[test]
    pub fn test_deserialize_complex_header_arrays() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'
body = 'hello'

[headers]
Accept = [
    { value = 'application/json', active = false, secret = false },
    { value = 'text/html', active = false, secret = false },
]
X-Client-Id = [
    { value = '123412341234', active = true, secret = true },
    { value = '{{CLIENT_ID}}', active = false, secret = false },
]
Accept-Encoding = 'gzip'
";
        let EndpointLoadResult::Success(endpoint) = super::read_endpoint_string(toml) else {
            panic!("wrong read");
        };
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                encoding: RawEncoding::OctetStream,
                content: Vec::from(b"hello"),
            }
        );
        assert_eq!(endpoint.headers.len(), 5);

        let mut given_headers = endpoint.headers.clone();
        given_headers.sort();
        assert_eq!(
            given_headers,
            KeyValueTable::new(&vec![
                KeyValue {
                    name: "Accept".into(),
                    value: "application/json".into(),
                    active: false,
                    secret: false,
                },
                KeyValue {
                    name: "Accept".into(),
                    value: "text/html".into(),
                    active: false,
                    secret: false,
                },
                KeyValue {
                    name: "Accept-Encoding".into(),
                    value: "gzip".into(),
                    active: true,
                    secret: false
                },
                KeyValue {
                    name: "X-Client-Id".into(),
                    value: "123412341234".into(),
                    active: true,
                    secret: true
                },
                KeyValue {
                    name: "X-Client-Id".into(),
                    value: "{{CLIENT_ID}}".into(),
                    active: false,
                    secret: false
                },
            ]),
        );
    }

    #[test]
    pub fn test_invalid_file_version() {
        let toml = "
version = 0
url = 'https://www.google.com'
method = 'GET'
body = 'hello'
";
        let EndpointLoadResult::Error(e) = super::read_endpoint_string(toml) else {
            panic!("expected a failure");
        };
        assert_eq!(e, FileLoadError::OutdatedSchema);
    }

    #[test]
    pub fn test_file_version_too_new() {
        let toml = "
version = 2
url = 'https://www.google.com'
method = 'GET'
body = 'hello'
";
        let EndpointLoadResult::Error(e) = super::read_endpoint_string(toml) else {
            panic!("expected a failure");
        };
        assert_eq!(e, FileLoadError::OutdatedSchema);
    }

    #[test]
    pub fn test_invalid_method() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'THROW'
";
        let EndpointLoadResult::Warning(endpoint, tags) = super::read_endpoint_string(toml) else {
            panic!("expected to load with warnings");
        };
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(endpoint.body, RequestPayload::None);
        assert_eq!(1, tags.len());
        assert_eq!(FileWarningTag::InvalidHttpVerb("THROW".into()), tags[0]);
    }

    #[test]
    pub fn test_empty_method() {
        let toml = "
version = 1
url = 'https://www.google.com'
body = 'hello'

[headers]
Accept = 'text/html'
";
        let EndpointLoadResult::Error(_) = super::read_endpoint_string(toml) else {
            panic!("expected a failure");
        };
    }

    #[test]
    pub fn test_empty_body() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'GET'

[headers]
Accept = 'text/html'
";
        let EndpointLoadResult::Success(endpoint) = super::read_endpoint_string(toml) else {
            panic!("wrong read");
        };
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Get);
        assert_eq!(endpoint.body, RequestPayload::None);
    }

    #[test]
    pub fn test_multiple_headers_serialization() {
        let headers = vec![
            ("Host", "google.com").into(),
            ("User-Agent", "Cartero").into(),
            ("User-Agent", "Cartero/0.1").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = RequestPayload::None;
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables: KeyValueTable::default(),
            body,
            parameters: KeyValueTable::default(),
        };

        let content = super::write_endpoint_string(&r).unwrap();
        let content = content.as_str();
        assert!(content.contains("url = \"https://www.google.com\""));
        assert!(content.contains("Host = \"google.com\""));
        assert!(content.contains("User-Agent = ["));
    }

    #[test]
    pub fn test_multiple_headers_serialization_with_meta() {
        let headers = vec![
            ("Host", "google.com").into(),
            ("User-Agent", "Cartero").into(),
            KeyValue {
                name: "User-Agent".into(),
                value: "Cartero/devel".into(),
                active: false,
                secret: false,
            },
            ("User-Agent", "Cartero/0.1").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = RequestPayload::None;
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables: KeyValueTable::default(),
            body,
            parameters: KeyValueTable::default(),
        };

        let content = super::write_endpoint_string(&r).unwrap();
        let content = content.as_str();
        assert!(content.contains("url = \"https://www.google.com\""));
        assert!(content.contains("Host = \"google.com\""));
        assert!(content.contains("User-Agent = ["));
        assert!(content.contains("active = false"));
    }

    #[test]
    pub fn test_empty_headers() {
        let toml = "
version = 1
url = 'https://www.google.com'
method = 'POST'
body = 'hello'
";
        let EndpointLoadResult::Success(endpoint) = super::read_endpoint_string(toml) else {
            panic!("wrong read");
        };
        assert_eq!(endpoint.url, "https://www.google.com");
        assert_eq!(endpoint.method, RequestMethod::Post);
        assert_eq!(
            endpoint.body,
            RequestPayload::Raw {
                content: Vec::from(b"hello"),
                encoding: RawEncoding::OctetStream,
            }
        );
        assert_eq!(endpoint.headers.len(), 0);
    }

    #[test]
    pub fn test_serialize_correctly() {
        let headers = vec![
            ("User-Agent", "Cartero").into(),
            ("Host", "google.com").into(),
        ];
        let headers = KeyValueTable::new(&headers);
        let body = RequestPayload::Raw {
            content: Vec::from(b"Hello"),
            encoding: RawEncoding::OctetStream,
        };
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables: KeyValueTable::default(),
            body,
            parameters: KeyValueTable::default(),
        };

        let content = super::write_endpoint_string(&r).unwrap();
        assert!(content
            .as_str()
            .contains("url = \"https://www.google.com\""));
        assert!(content.as_str().contains("method = \"POST\""));
        assert!(content.as_str().contains("body = \"Hello\""));
        assert!(content.as_str().contains("User-Agent = \"Cartero\""));
    }

    #[test]
    pub fn test_serializes_complex_example() {
        // One thing important to test: since this is eventually a hashmap, the result
        // will be sorted by key name, but the order of the elements must match the
        // original order.
        let headers = KeyValueTable::new(&vec![
            KeyValue {
                name: "X-Client-Id".into(),
                value: "123412341234".into(),
                secret: true,
                active: true,
            },
            ("Host", "google.com").into(),
            ("User-Agent", "Cartero").into(),
            KeyValue {
                name: "X-Client-Id".into(),
                value: "{{CLIENT_ID}}".into(),
                secret: false,
                active: false,
            },
        ]);
        let variables = KeyValueTable::new(&[
            KeyValue {
                name: "CLIENT_SECRET".into(),
                value: "101010".into(),
                secret: true,
                active: true,
            },
            ("CLIENT_ID", "123412341234").into(),
            KeyValue {
                name: "CLIENT_SECRET".into(),
                value: "202020".into(),
                secret: true,
                active: true,
            },
        ]);
        let body = RequestPayload::Raw {
            content: Vec::from(b"Hello"),
            encoding: RawEncoding::OctetStream,
        };
        let r = EndpointData {
            url: "https://www.google.com".to_string(),
            method: RequestMethod::Post,
            headers,
            variables,
            body,
            parameters: KeyValueTable::default(),
        };

        let content = super::write_endpoint_string(&r).unwrap();
        let EndpointLoadResult::Success(parsed) = super::read_endpoint_string(&content) else {
            panic!("wrong read");
        };
        assert_eq!(r.url, parsed.url);
        assert_eq!(r.method, parsed.method);
        assert_eq!(r.body, parsed.body);

        assert_eq!(
            KeyValueTable::new(&vec![
                r.headers[1].clone(),
                r.headers[2].clone(),
                r.headers[0].clone(),
                r.headers[3].clone(),
            ]),
            parsed.headers
        );
        assert_eq!(
            KeyValueTable::new(&[
                r.variables[1].clone(),
                r.variables[0].clone(),
                r.variables[2].clone()
            ]),
            parsed.variables
        );
    }
}
