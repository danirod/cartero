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

    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate};

    use crate::widgets::{BasePayloadPane, BasePayloadPaneImpl, KeyValuePane};

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::UrlencodedPayloadPane)]
    #[template(resource = "/es/danirod/Cartero/urlencoded_payload_pane.ui")]
    pub struct UrlencodedPayloadPane {
        #[template_child]
        data: TemplateChild<KeyValuePane>,

        #[property(get = Self::payload, set = Self::set_payload, nullable, type = Option<glib::Bytes>)]
        _payload: RefCell<Option<glib::Bytes>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UrlencodedPayloadPane {
        const NAME: &'static str = "CarteroUrlencodedPayloadPane";
        type Type = super::UrlencodedPayloadPane;
        type ParentType = BasePayloadPane;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for UrlencodedPayloadPane {
        fn constructed(&self) {
            self.parent_constructed();
            self.data.assert_always_placeholder();
        }
    }

    impl WidgetImpl for UrlencodedPayloadPane {}

    impl BasePayloadPaneImpl for UrlencodedPayloadPane {}

    impl UrlencodedPayloadPane {
        pub fn payload(&self) -> Option<glib::Bytes> {
            let entries = self.data.get_entries();
            let variables: Vec<(String, String)> = entries
                .iter()
                .filter(|entry| entry.is_usable())
                .map(|entry| (entry.header_name(), entry.header_value()))
                .collect();
            match serde_urlencoded::to_string(variables) {
                Ok(cod) => {
                    let bytes = glib::Bytes::from_owned(cod);
                    Some(bytes)
                }
                Err(_) => None,
            }
        }

        pub fn set_payload(&self, _: Option<&glib::Bytes>) {
            // NOOP
        }
    }
}

glib::wrapper! {
    pub struct UrlencodedPayloadPane(ObjectSubclass<imp::UrlencodedPayloadPane>)
        @extends gtk::Widget, BasePayloadPane,
    @implements gtk::Accessible, gtk::Buildable;
}
