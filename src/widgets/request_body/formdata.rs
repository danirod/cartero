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

use super::BasePayloadPane;

mod imp {
    use std::cell::RefCell;
    use std::io::BufWriter;
    use std::io::Write;

    use formdata::FormData;
    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate};

    use crate::entities::KeyValue;
    use crate::entities::KeyValueTable;
    use crate::widgets::{BasePayloadPane, BasePayloadPaneImpl, KeyValuePane};

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::FormdataPayloadPane)]
    #[template(resource = "/es/danirod/Cartero/formdata_payload_pane.ui")]
    pub struct FormdataPayloadPane {
        #[template_child]
        data: TemplateChild<KeyValuePane>,

        #[property(get = Self::payload, set = Self::set_payload, nullable, type = Option<glib::Bytes>)]
        _payload: RefCell<Option<glib::Bytes>>,

        #[property(get = Self::headers, type = KeyValueTable)]
        _headers: RefCell<KeyValueTable>,

        #[property(get, set)]
        boundary: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FormdataPayloadPane {
        const NAME: &'static str = "CarteroFormdataPayloadPane";
        type Type = super::FormdataPayloadPane;
        type ParentType = BasePayloadPane;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FormdataPayloadPane {
        fn constructed(&self) {
            self.parent_constructed();
            self.data.assert_always_placeholder();

            let boundary = formdata::generate_boundary();
            let boundary = String::from_utf8_lossy(&boundary).to_string();
            self.boundary.set(boundary);
        }
    }

    impl WidgetImpl for FormdataPayloadPane {}

    impl BasePayloadPaneImpl for FormdataPayloadPane {}

    impl FormdataPayloadPane {
        pub fn payload(&self) -> Option<glib::Bytes> {
            let entries = self.data.get_entries();
            let variables: Vec<(String, String)> = entries
                .iter()
                .filter(|entry| entry.is_usable())
                .map(|entry| (entry.header_name(), entry.header_value()))
                .collect();

            // Build response
            let data = FormData {
                fields: variables,
                files: vec![],
            };
            let boundary = self.boundary.borrow();
            let boundary = Vec::from(boundary.as_bytes());
            let mut stream = BufWriter::new(Vec::new());
            let _ = formdata::write_formdata(&mut stream, &boundary, &data);

            let _ = stream.flush();
            let copy = stream.get_ref().clone();
            let contents = glib::Bytes::from(&copy);
            Some(contents)
        }

        pub fn set_payload(&self, _: Option<&glib::Bytes>) {
            // NOOP
        }

        pub fn headers(&self) -> KeyValueTable {
            let boundary = self.boundary.borrow();
            let content_type = format!("multipart/form-data; boundary={boundary}");
            let content_type = KeyValue::from(("Content-Type", content_type.as_str()));
            KeyValueTable::new(&[content_type])
        }
    }
}

glib::wrapper! {
    pub struct FormdataPayloadPane(ObjectSubclass<imp::FormdataPayloadPane>)
        @extends gtk::Widget, BasePayloadPane,
    @implements gtk::Accessible, gtk::Buildable;
}
