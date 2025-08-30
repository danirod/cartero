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

use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use cartero_objects::{Request, RequestBodyDataExt, RequestBodyType};
use gio::prelude::FileExt;

use crate::{active_pairs, RequestEnvironment, RequestError};

#[derive(Default)]
pub(crate) struct BoundBody {
    content: Option<Vec<u8>>,
    headers: Vec<(String, String)>,
}

impl BoundBody {
    pub fn headers(&self) -> Vec<(String, String)> {
        self.headers.clone()
    }

    pub fn body(&self) -> Option<Vec<u8>> {
        self.content.clone()
    }
    pub async fn new(value: &Request, env: &RequestEnvironment) -> Result<Self, RequestError> {
        let value = value.resolve()?;

        let body_type = value.body().body_type();

        match body_type {
            RequestBodyType::None => Ok(BoundBody::default()),
            RequestBodyType::UrlEncoded => Self::try_from_urlencoded(&value),
            RequestBodyType::Multipart => Self::try_from_multipart(&value),
            RequestBodyType::Raw => Self::try_from_raw(&value),
            RequestBodyType::File => Self::try_from_file(&value, env).await,
        }
    }

    async fn try_from_file(
        value: &Request,
        env: &RequestEnvironment,
    ) -> Result<Self, RequestError> {
        let file = value.body().file().unwrap();
        let desired_path = file.path();
        if desired_path.trim().is_empty() {
            return Ok(Self::default());
        }
        let Some(prefix) = env.prefix.clone() else {
            return Err(RequestError::FilePrefixUnset);
        };

        // Must be a relative path according to the security rules.
        // (So, no absolute paths and no files that are outside of prefix dir).
        let desired_pathbuf: PathBuf = desired_path.as_str().into();
        let absolute_file = prefix.resolve_relative_path(&desired_pathbuf);
        if !absolute_file.has_prefix(&prefix) {
            return Err(RequestError::UnsecureFile(desired_path));
        }

        // Generate the output
        match absolute_file.load_bytes_future().await {
            Ok((content, _)) => Ok(Self {
                content: Some(content.to_vec()),
                headers: file.rendered_headers(),
            }),
            Err(e) => Err(RequestError::IOError(Box::new(e))),
        }
    }

    fn try_from_urlencoded(value: &Request) -> Result<Self, RequestError> {
        let urlencoded = value.body().urlencoded().unwrap();

        let values = urlencoded.params();
        let pairs = active_pairs(&values);
        let body = serde_urlencoded::to_string(pairs).map_err(|_| RequestError::EncodingError)?;
        let raw = Vec::from(body.as_str());

        Ok(Self {
            headers: urlencoded.rendered_headers(),
            content: Some(raw),
        })
    }

    fn try_from_multipart(value: &Request) -> Result<Self, RequestError> {
        let multipart = value.body().multipart().unwrap();
        let values = multipart.params();
        let pairs = active_pairs(&values);

        let boundary = formdata::generate_boundary();
        let formdata = formdata::FormData {
            fields: pairs,
            // TODO: as soon as we have collections, add files
            files: vec![],
        };
        let mut stream = BufWriter::new(Vec::new());
        formdata::write_formdata(&mut stream, &boundary, &formdata).map_err(|e| {
            glib::g_error!("cartero", "form data error: {}", e);
            RequestError::EncodingError
        })?;
        stream.flush().map_err(|_| RequestError::EncodingError)?;

        let body = stream.get_ref().clone();
        let content_type = format!(
            "multipart/form-data; boundary={}",
            String::from_utf8_lossy(&boundary)
        );
        let headers = vec![("Content-Type".into(), content_type)];
        Ok(Self {
            headers,
            content: Some(body),
        })
    }

    fn try_from_raw(value: &Request) -> Result<Self, RequestError> {
        let raw = value.body().raw().unwrap();
        let content = raw.payload();
        Ok(Self {
            content: Some(Vec::from(content.as_str())),
            headers: raw.rendered_headers(),
        })
    }
}

#[cfg(test)]
mod tests {
    use cartero_objects::*;

    use super::*;

    fn dummy_env() -> RequestEnvironment {
        RequestEnvironment {
            prefix: None,
            env_file: None,
            config: crate::ClientConfig {
                validate_tls: false,
                redirects: 0,
                timeout: 30.0,
            },
        }
    }

    #[tokio::test]
    async fn test_urlencoded() {
        let field_table = FieldTable::from_iter(vec![
            Field::builder().key("category").value("10").build(),
            Field::builder()
                .key("user_id")
                .value("200")
                .active(false)
                .build(),
            Field::builder().key("superuser").value("true").build(),
        ]);

        let urlenc = RequestBodyUrlencoded::builder()
            .params(&field_table)
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().urlencoded(&urlenc).build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/x-www-form-urlencoded", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"category=10&superuser=true",
            result.content.unwrap().as_slice()
        );
    }

    #[tokio::test]
    async fn test_urlencoded_with_variables() {
        let field_table = FieldTable::from_iter(vec![
            Field::builder()
                .key("category")
                .value("{{CATEGORY}}")
                .build(),
            Field::builder()
                .key("user_id")
                .value("200")
                .active(false)
                .build(),
            Field::builder().key("superuser").value("true").build(),
        ]);

        let urlenc = RequestBodyUrlencoded::builder()
            .params(&field_table)
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().urlencoded(&urlenc).build())
            .variable(&Field::builder().key("CATEGORY").value("30").build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/x-www-form-urlencoded", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"category=30&superuser=true",
            result.content.unwrap().as_slice()
        );
    }

    #[tokio::test]
    async fn test_multipart() {
        let field_table = FieldTable::from_iter(vec![
            Field::builder().key("category").value("10").build(),
            Field::builder()
                .key("user_id")
                .value("200")
                .active(false)
                .build(),
            Field::builder().key("superuser").value("true").build(),
        ]);

        let mpart = RequestBodyMultipart::builder().params(&field_table).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().multipart(&mpart).build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);

        assert!(result.headers[0]
            .1
            .starts_with("multipart/form-data; boundary="));
        let boundary = result.headers[0]
            .1
            .replace("multipart/form-data; boundary=", "");

        let Some(content) = result.content else {
            panic!("expected some content");
        };
        let body = String::from_utf8_lossy(&content);
        let expected = {
            let lines = vec![
                format!("--{boundary}"),
                "Content-Type: text/plain".into(),
                "Content-Disposition: form-data; name=\"category\"".into(),
                "".into(),
                "10".into(),
                format!("--{boundary}"),
                "Content-Type: text/plain".into(),
                "Content-Disposition: form-data; name=\"superuser\"".into(),
                "".into(),
                "true".into(),
                format!("--{boundary}--"),
            ];
            lines.join("\r\n")
        };
        assert_eq!(expected, body);
    }

    #[tokio::test]
    async fn test_multipart_with_variables() {
        let field_table = FieldTable::from_iter(vec![
            Field::builder()
                .key("category")
                .value("{{CATEGORY}}")
                .build(),
            Field::builder()
                .key("user_id")
                .value("200")
                .active(false)
                .build(),
            Field::builder().key("superuser").value("true").build(),
        ]);

        let mpart = RequestBodyMultipart::builder().params(&field_table).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().multipart(&mpart).build())
            .variable(&Field::builder().key("CATEGORY").value("30").build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);

        assert!(result.headers[0]
            .1
            .starts_with("multipart/form-data; boundary="));
        let boundary = result.headers[0]
            .1
            .replace("multipart/form-data; boundary=", "");

        let Some(content) = result.content else {
            panic!("expected some content");
        };
        let body = String::from_utf8_lossy(&content);
        let expected = {
            let lines = vec![
                format!("--{boundary}"),
                "Content-Type: text/plain".into(),
                "Content-Disposition: form-data; name=\"category\"".into(),
                "".into(),
                "30".into(),
                format!("--{boundary}"),
                "Content-Type: text/plain".into(),
                "Content-Disposition: form-data; name=\"superuser\"".into(),
                "".into(),
                "true".into(),
                format!("--{boundary}--"),
            ];
            lines.join("\r\n")
        };
        assert_eq!(expected, body);
    }

    #[tokio::test]
    async fn test_octet_stream() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
            .payload("the payload")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&raw).build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/octet-stream", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(b"the payload", result.content.unwrap().as_slice());
    }

    #[tokio::test]
    async fn test_octet_stream_with_variables() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::OctetStream)
            .payload("Hello {{NAME}}!")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&raw).build())
            .variable(&Field::builder().key("NAME").value("world").build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/octet-stream", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(b"Hello world!", result.content.unwrap().as_slice());
    }

    #[tokio::test]
    async fn test_xml() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::Xml)
            .payload("<?xml version=\"1.0\" ?><hello who=\"world\" />")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&raw).build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/xml", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"<?xml version=\"1.0\" ?><hello who=\"world\" />",
            result.content.unwrap().as_slice()
        );
    }

    #[tokio::test]
    async fn test_xml_with_variables() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::Xml)
            .payload("<?xml version=\"1.0\" ?><hello who=\"{{USER}}\" />")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&raw).build())
            .variable(&Field::builder().key("USER").value("world").build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/xml", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"<?xml version=\"1.0\" ?><hello who=\"world\" />",
            result.content.unwrap().as_slice()
        );
    }

    #[tokio::test]
    async fn test_json() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::Json)
            .payload("{\"hello\": \"world\"}")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&raw).build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/json", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"{\"hello\": \"world\"}",
            result.content.unwrap().as_slice()
        );
    }

    #[tokio::test]
    async fn test_json_with_variables() {
        let raw = RequestBodyRaw::builder(RequestBodyRawType::Json)
            .payload("{\"hello\": \"{{USER}}\"}")
            .build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(RequestBody::builder().raw(&raw).build())
            .variable(&Field::builder().key("USER").value("world").build())
            .build();

        let env = dummy_env();
        let result = BoundBody::new(&req, &env).await.unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/json", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"{\"hello\": \"world\"}",
            result.content.unwrap().as_slice()
        );
    }

    fn gio_file_for_current_file() -> gio::File {
        let current_file = file!();
        // TODO: WHY DO I EVEN HAVE TO DO THIS, I DON'T HAVE AN EXPLANATION FOR THIS
        // AND I'M GOING SKIZO. Investigate why the path is being prefixed twice.
        let (_, current_file) = current_file.split_once("/").unwrap();
        gio::File::for_path(current_file)
    }

    #[tokio::test]
    async fn test_file() {
        let file = RequestBodyFile::builder()
            .path("fixtures/hello.txt")
            .content_type(Some("text/plain"))
            .build();
        let body = RequestBody::builder().file(&file).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(body)
            .build();

        let env = dummy_env();
        let env = RequestEnvironment {
            config: env.config,
            env_file: None,
            prefix: gio_file_for_current_file().parent(),
        };

        let result = glib::MainContext::new()
            .block_on(async { BoundBody::new(&req, &env).await })
            .unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("text/plain", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"this is the file contents\n",
            result.content.unwrap().as_slice()
        );
    }

    #[tokio::test]
    async fn test_file_for_file_that_does_not_exist() {
        let file = RequestBodyFile::builder()
            .path("fixtures/not_found.txt")
            .content_type(Some("text/plain"))
            .build();
        let body = RequestBody::builder().file(&file).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(body)
            .build();

        let env = dummy_env();
        let env = RequestEnvironment {
            config: env.config,
            env_file: None,
            prefix: gio_file_for_current_file().parent(),
        };

        let result = glib::MainContext::new().block_on(async { BoundBody::new(&req, &env).await });

        match result {
            Err(RequestError::IOError(_)) => {}
            _ => panic!("Expected an IOError for this one"),
        }
    }

    #[tokio::test]
    async fn test_file_for_file_that_is_not_a_file() {
        let file = RequestBodyFile::builder()
            .path("fixtures")
            .content_type(Some("text/plain"))
            .build();
        let body = RequestBody::builder().file(&file).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(body)
            .build();

        let env = dummy_env();
        let env = RequestEnvironment {
            config: env.config,
            env_file: None,
            prefix: gio_file_for_current_file().parent(),
        };

        let result = glib::MainContext::new().block_on(async { BoundBody::new(&req, &env).await });

        match result {
            Err(RequestError::IOError(_)) => {}
            _ => panic!("Expected an IOError for this one"),
        }
    }

    #[tokio::test]
    async fn test_file_outside_current_dir() {
        let file = RequestBodyFile::builder()
            .path("../testing.txt")
            .content_type(Some("text/plain"))
            .build();
        let body = RequestBody::builder().file(&file).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(body)
            .build();

        let env = dummy_env();
        let env = RequestEnvironment {
            config: env.config,
            env_file: None,
            prefix: gio_file_for_current_file().parent(),
        };

        let result = glib::MainContext::new().block_on(async { BoundBody::new(&req, &env).await });

        match result {
            Err(RequestError::UnsecureFile(_)) => {}
            _ => panic!("Expected an UnsecureFile for this one"),
        }
    }

    #[tokio::test]
    async fn test_file_absolute_dir() {
        let file = RequestBodyFile::builder()
            .path("/etc/passwd")
            .content_type(Some("text/plain"))
            .build();
        let body = RequestBody::builder().file(&file).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(body)
            .build();

        let env = dummy_env();
        let env = RequestEnvironment {
            config: env.config,
            env_file: None,
            prefix: gio_file_for_current_file().parent(),
        };

        let result = glib::MainContext::new().block_on(async { BoundBody::new(&req, &env).await });

        match result {
            Err(RequestError::UnsecureFile(_)) => {}
            _ => panic!("Expected an UnsecureFile for this one"),
        }
    }

    #[tokio::test]
    async fn test_file_without_a_prefix() {
        let file = RequestBodyFile::builder()
            .path("fixtures/hello.txt")
            .content_type(Some("text/plain"))
            .build();
        let body = RequestBody::builder().file(&file).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(body)
            .build();

        let env = dummy_env();
        let env = RequestEnvironment {
            config: env.config,
            env_file: None,
            prefix: None,
        };

        let result = glib::MainContext::new().block_on(async { BoundBody::new(&req, &env).await });

        match result {
            Err(RequestError::FilePrefixUnset) => {}
            _ => panic!("Expected an UnsecureFile for this one"),
        }
    }

    #[tokio::test]
    async fn test_file_with_default_content_type() {
        let file = RequestBodyFile::builder()
            .path("fixtures/hello.txt")
            .build();
        let body = RequestBody::builder().file(&file).build();
        let req = Request::builder("https://www.example.com", RequestMethod::Get)
            .with_body(body)
            .build();

        let env = dummy_env();
        let env = RequestEnvironment {
            config: env.config,
            env_file: None,
            prefix: gio_file_for_current_file().parent(),
        };

        let result = glib::MainContext::new()
            .block_on(async { BoundBody::new(&req, &env).await })
            .unwrap();

        assert_eq!(1, result.headers.len());
        assert_eq!("Content-Type", result.headers[0].0);
        assert_eq!("application/octet-stream", result.headers[0].1);

        assert!(result.content.is_some());
        assert_eq!(
            b"this is the file contents\n",
            result.content.unwrap().as_slice()
        );
    }
}
