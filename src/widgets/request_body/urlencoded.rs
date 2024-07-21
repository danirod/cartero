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

use glib::subclass::types::ObjectSubclassIsExt;

use crate::entities::RequestPayload;

use super::{BasePayloadPane, BasePayloadPaneExt};

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use crate::entities::{KeyValue, KeyValueTable};
    use crate::objects::KeyValueItem;
    use crate::widgets::{BasePayloadPane, BasePayloadPaneImpl, KeyValuePane};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/urlencoded_payload_pane.ui")]
    pub struct UrlencodedPayloadPane {
        #[template_child]
        data: TemplateChild<KeyValuePane>,
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

    impl ObjectImpl for UrlencodedPayloadPane {
        fn constructed(&self) {
            self.parent_constructed();
            self.data.assert_always_placeholder();
        }
    }

    impl WidgetImpl for UrlencodedPayloadPane {}

    impl BasePayloadPaneImpl for UrlencodedPayloadPane {}

    impl UrlencodedPayloadPane {
        pub(super) fn get_table(&self) -> KeyValueTable {
            let entries = self.data.get_entries();
            let key_values: Vec<KeyValue> = entries.into_iter().map(KeyValue::from).collect();
            KeyValueTable::new(&key_values)
        }

        pub(super) fn set_table(&self, table: &KeyValueTable) {
            let key_values: Vec<KeyValueItem> = table
                .iter()
                .map(|row| KeyValueItem::from(row.clone()))
                .collect();
            self.data.set_entries(&key_values);
        }
    }
}

glib::wrapper! {
    pub struct UrlencodedPayloadPane(ObjectSubclass<imp::UrlencodedPayloadPane>)
        @extends gtk::Widget, BasePayloadPane,
    @implements gtk::Accessible, gtk::Buildable;
}

impl BasePayloadPaneExt for UrlencodedPayloadPane {
    fn payload(&self) -> RequestPayload {
        let imp = self.imp();
        let table = imp.get_table();
        RequestPayload::Urlencoded(table)
    }

    fn set_payload(&self, payload: &RequestPayload) {
        let imp = self.imp();
        if let RequestPayload::Urlencoded(params) = payload {
            imp.set_table(params);
        }
    }
}
