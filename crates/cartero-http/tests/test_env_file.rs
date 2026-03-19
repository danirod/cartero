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

use std::path::PathBuf;

use cartero_http::{BoundRequest, ClientConfig, RequestEnvironment};
use cartero_objects::{
    EnvFile, Field, Request, RequestAuthentication, RequestAuthenticationBearer,
};

#[tokio::test]
async fn test_read_env_vars() {
    let request = Request::builder("{{HOST_URL}}/users", cartero_objects::RequestMethod::Get)
        .header(
            &Field::builder()
                .key("X-Client-Id")
                .value("{{CLIENT_ID}}")
                .build(),
        )
        .with_auth(
            RequestAuthentication::builder()
                .bearer_token(
                    &RequestAuthenticationBearer::builder()
                        .token("{{API_KEY}}")
                        .build(),
                )
                .build(),
        )
        .build();

    let env_file_src = gio::File::for_path(PathBuf::from("tests/.env"));
    let env_file = EnvFile::builder().file(Some(&env_file_src)).build();
    let env = RequestEnvironment {
        prefix: None,
        proxy: None,
        env_file: Some(env_file),
        config: ClientConfig {
            validate_tls: false,
            redirects: 0,
            timeout: 30.0,
        },
    };

    let bound = BoundRequest::new(&request, &env).await.unwrap();
    assert_eq!(bound.url, "https://staging.example.com/users");
    assert_eq!(bound.headers.len(), 2);
    assert_eq!(bound.headers["X-Client-Id"], "client-123412341234");
    assert_eq!(bound.headers["Authorization"], "Bearer 1234123412341234")
}
