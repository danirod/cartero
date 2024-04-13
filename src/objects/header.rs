// Copyright 2024 the Cartero authors
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

use glib::Object;

mod imp {
    use std::cell::RefCell;

    use glib::Properties;
    use gtk4::glib;
    use gtk4::glib::prelude::*;
    use gtk4::glib::subclass::prelude::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type = super::Header)]
    pub struct Header {
        #[property(get, set)]
        active: RefCell<bool>,
        #[property(get, set)]
        header_name: RefCell<String>,
        #[property(get, set)]
        header_value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Header {
        const NAME: &'static str = "CarteroHeader";
        type Type = super::Header;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Header {}
}

glib::wrapper! {
    pub struct Header(ObjectSubclass<imp::Header>);
}

impl Default for Header {
    fn default() -> Self {
        Object::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use super::Header;

    #[test]
    pub fn test_header_properties() {
        let header = Header::default();
        assert_eq!(header.header_name(), "");
        assert_eq!(header.header_value(), "");
        assert!(!header.active());

        header.set_header_name("Content-Type");
        header.set_header_value("text/plain");
        header.set_active(true);
        assert_eq!(header.header_name(), "Content-Type");
        assert_eq!(header.header_value(), "text/plain");
        assert!(header.active());

        header.set_header_name("Accept");
        assert_eq!(header.header_name(), "Accept");
        assert_eq!(header.header_value(), "text/plain");
        assert!(header.active());
    }
}
