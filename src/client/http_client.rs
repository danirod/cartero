use std::io::Read;
use std::str::FromStr;

use isahc::http::{HeaderName, HeaderValue, Method as IsaMethod};
use isahc::{Body, Request as IsaRequest, Response as IsaResponse};

use super::{Request, RequestError, RequestMethod, Response};

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

impl TryFrom<Request> for isahc::Request<Vec<u8>> {
    type Error = RequestError;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        let mut builder = IsaRequest::builder().uri(&req.url).method(&req.method);
        {
            let Some(headers) = builder.headers_mut() else {
                return Err(RequestError);
            };
            for (h, v) in &req.headers {
                let key = HeaderName::from_str(&h)?;
                let value = HeaderValue::from_str(&v)?;
                headers.insert(key, value);
            }
        }
        let req = builder.body(req.body.clone())?;
        Ok(req)
    }
}

impl TryFrom<&mut IsaResponse<Body>> for Response {
    type Error = RequestError;

    fn try_from(value: &mut IsaResponse<Body>) -> Result<Self, Self::Error> {
        let status_code: u16 = value.status().as_u16();
        let headers = value
            .headers()
            .iter()
            .map(|(k, v)| {
                let header_name = k.to_string();
                let header_value = String::from(v.to_str().unwrap());
                (header_name, header_value)
            })
            .collect();
        let body = {
            let mut buffer = Vec::new();
            let body = value.body_mut();
            body.read_to_end(&mut buffer)?;
            buffer
        };
        Ok(Response {
            duration: 0,
            size: 0,
            status_code,
            headers,
            body,
        })
    }
}
