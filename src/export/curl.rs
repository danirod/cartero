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

use cartero_objects::{Request, RequestBodyRawType};
use serde_json::{Error, Value};

pub struct CodeExportService {
    request: Request,
}

impl CodeExportService {
    pub fn new(request: Request) -> Self {
        Self { request }
    }

    pub fn generate(&self) -> Result<String, cartero_http::RequestError> {
        let bound_request = cartero_http::BoundRequest::try_from(self.request.clone())?;
        let mut command = "curl".to_string();

        command.push_str(&{
            let method_str: String = bound_request.method.to_string();
            format!(" -X {} '{}'", method_str, bound_request.url)
        });

        if !bound_request.headers.is_empty() {
            let size = bound_request.headers.len();
            let mut keys: Vec<&String> = bound_request.headers.keys().collect();
            keys.sort();

            command.push_str(" \\\n");

            for (i, key) in keys.iter().enumerate() {
                let val = bound_request.headers.get(*key).unwrap();

                command.push_str(&{
                    let mut initial = format!("  -H '{key}: {val}'");

                    if i < size - 1 {
                        initial.push_str(" \\\n");
                    }

                    initial
                });
            }
        }

        if let Some(_) = self.request.body().urlencoded() {
            if let Some(bd) = bound_request.body.clone() {
                let str = String::from_utf8_lossy(&bd).to_string();
                command.push_str(&format!(" \\\n  -d '{str}'"));
            }
        }

        if let Some(raw) = self.request.body().raw() {
            let content = bound_request.body.clone().unwrap_or_default();
            let encoding = raw.payload_type();
            match encoding {
                RequestBodyRawType::Json => {
                    command.push_str(&'fmt: {
                        let body = String::from_utf8_lossy(&content).to_string();
                        let value: Result<Value, Error> = serde_json::from_str(body.as_ref());

                        if value.is_err() {
                            break 'fmt String::new();
                        }

                        let value = value.unwrap();
                        let trimmed_json_str = serde_json::to_string(&value);

                        if trimmed_json_str.is_err() {
                            break 'fmt String::new();
                        }

                        let trimmed_json_str = trimmed_json_str.unwrap();
                        let trimmed_json_str = trimmed_json_str.replace("'", "\\\\'");

                        format!(" \\\n  -d '{}'", trimmed_json_str)
                    });
                }
                _ => {
                    command.push_str(&{
                        let string = String::from_utf8_lossy(&content).to_string();
                        format!(" \\\n  -d '{}'", string)
                    });
                }
            }
        }

        Ok(command)
    }
}
