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

/// The HTTP verb in use during an HTTP request.
///
/// The actual list of verbs may be larger, but these are the ones we
/// currently support. Each one is linked to a verb used during an HTTP
/// request, and can be picked from the dropdown in the user interface.
///
/// Each verb can be converted to a string or string-like object using the
/// `.as_ref()` function, because the trait `AsRef<str>` is implemented,
/// allowing to do things such as:
///
/// ```
/// use cartero_objects::RequestMethod;
///
/// let verb = RequestMethod::Post;
/// let str = verb.to_string();
/// assert_eq!("POST", str);
/// ```
///
/// Additionally, the `TryFrom<&'a str>` trait is implemented, allowing to
/// try-cast a verb into a variant of this enum. If the verb is not supported,
/// an error is returned.
///
/// ```
/// use cartero_objects::RequestMethod;
///
/// let try_get = RequestMethod::try_from("GET");
/// assert!(try_get.is_ok_and(|verb| verb == RequestMethod::Get));
///
/// let try_harder = RequestMethod::try_from("HARDER");
/// assert!(try_harder.is_err());
/// ```
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroRequestMethod")]
pub enum RequestMethod {
    /// GET request.
    #[default]
    #[enum_value(name = "GET")]
    Get,

    /// POST request.
    #[enum_value(name = "POST")]
    Post,

    /// PUT request.
    #[enum_value(name = "PUT")]
    Put,

    /// PATCH request.
    #[enum_value(name = "PATCH")]
    Patch,

    /// DELETE request.
    #[enum_value(name = "DELETE")]
    Delete,

    /// OPTIONS request.
    #[enum_value(name = "OPTIONS")]
    Options,

    /// HEAD request.
    #[enum_value(name = "HEAD")]
    Head,

    /// TRACE request.
    #[enum_value(name = "TRACE")]
    Trace,
}

impl AsRef<str> for RequestMethod {
    fn as_ref(&self) -> &str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
        }
    }
}

impl std::fmt::Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl<'a> TryFrom<&'a str> for RequestMethod {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "patch" => Ok(Self::Patch),
            "delete" => Ok(Self::Delete),
            "options" => Ok(Self::Options),
            "head" => Ok(Self::Head),
            "trace" => Ok(Self::Trace),
            _ => Err(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_method_from_str() {
        let matches = vec![
            ("GET", RequestMethod::Get),
            ("Put", RequestMethod::Put),
            ("delete", RequestMethod::Delete),
            ("OpTiOnS", RequestMethod::Options),
        ];
        for (verb, expected) in matches {
            let value = RequestMethod::try_from(verb).unwrap();
            assert_eq!(value, expected);
        }

        let failing = RequestMethod::try_from("HELLO");
        assert!(failing.is_err());
    }

    #[test]
    fn test_request_method_to_str() {
        let matches = vec![
            (RequestMethod::Get, "GET"),
            (RequestMethod::Put, "PUT"),
            (RequestMethod::Trace, "TRACE"),
            (RequestMethod::Head, "HEAD"),
        ];
        for (verb, expected) in matches {
            let result = verb.as_ref();
            assert_eq!(result, expected);
        }
    }
}
