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

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum RequestMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
    Trace,
}

impl TryFrom<&str> for RequestMethod {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "get" => Ok(RequestMethod::Get),
            "post" => Ok(RequestMethod::Post),
            "put" => Ok(RequestMethod::Put),
            "patch" => Ok(RequestMethod::Patch),
            "delete" => Ok(RequestMethod::Delete),
            "options" => Ok(RequestMethod::Options),
            "head" => Ok(RequestMethod::Head),
            "trace" => Ok(RequestMethod::Trace),
            _ => Err(()),
        }
    }
}

impl From<RequestMethod> for &str {
    fn from(val: RequestMethod) -> Self {
        match val {
            RequestMethod::Get => "GET",
            RequestMethod::Post => "POST",
            RequestMethod::Put => "PUT",
            RequestMethod::Patch => "PATCH",
            RequestMethod::Delete => "DELETE",
            RequestMethod::Head => "HEAD",
            RequestMethod::Options => "OPTIONS",
            RequestMethod::Trace => "TRACE",
        }
    }
}

impl From<RequestMethod> for String {
    fn from(value: RequestMethod) -> String {
        let string: &str = value.into();
        String::from(string)
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::RequestMethod;

    #[test]
    pub fn test_convert_str_to_method() {
        assert!(RequestMethod::try_from("GET").is_ok_and(|x| x == RequestMethod::Get));
        assert!(RequestMethod::try_from("post").is_ok_and(|x| x == RequestMethod::Post));
        assert!(RequestMethod::try_from("Patch").is_ok_and(|x| x == RequestMethod::Patch));
        assert!(RequestMethod::try_from("Juan").is_err());
    }
}

impl Into<cartero_objects::RequestMethod> for RequestMethod {
    fn into(self) -> cartero_objects::RequestMethod {
        match self {
            RequestMethod::Get => cartero_objects::RequestMethod::Get,
            RequestMethod::Post => cartero_objects::RequestMethod::Post,
            RequestMethod::Put => cartero_objects::RequestMethod::Put,
            RequestMethod::Patch => cartero_objects::RequestMethod::Patch,
            RequestMethod::Delete => cartero_objects::RequestMethod::Delete,
            RequestMethod::Head => cartero_objects::RequestMethod::Head,
            RequestMethod::Options => cartero_objects::RequestMethod::Options,
            RequestMethod::Trace => cartero_objects::RequestMethod::Trace,
        }
    }
}

impl From<cartero_objects::RequestMethod> for RequestMethod {
    fn from(value: cartero_objects::RequestMethod) -> Self {
        match value {
            cartero_objects::RequestMethod::Get => Self::Get,
            cartero_objects::RequestMethod::Post => Self::Post,
            cartero_objects::RequestMethod::Put => Self::Put,
            cartero_objects::RequestMethod::Patch => Self::Patch,
            cartero_objects::RequestMethod::Delete => Self::Delete,
            cartero_objects::RequestMethod::Head => Self::Head,
            cartero_objects::RequestMethod::Options => Self::Options,
            cartero_objects::RequestMethod::Trace => Self::Trace,
        }
    }
}
