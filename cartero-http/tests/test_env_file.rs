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
