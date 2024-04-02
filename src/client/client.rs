use isahc::http::{Error, Method as IsaMethod};
use isahc::Request as IsaRequest;

use super::{Request, RequestMethod};

impl From<&RequestMethod> for isahc::http::Method {
    fn from(value: &RequestMethod) -> Self {
        match value {
            RequestMethod::Head => IsaMethod::HEAD,
            RequestMethod::Get => IsaMethod::GET,
            RequestMethod::Post => IsaMethod::POST,
            RequestMethod::Put => IsaMethod::PUT,
            RequestMethod::Patch => IsaMethod::PATCH,
            RequestMethod::Options => IsaMethod::OPTIONS,
            RequestMethod::Delete => IsaMethod::DELETE,
        }
    }
}

pub fn build_request(req: &Request) -> Result<IsaRequest<String>, Error> {
    IsaRequest::builder()
        .uri(&req.url)
        .method(&req.method)
        .body(req.body.clone())
}
