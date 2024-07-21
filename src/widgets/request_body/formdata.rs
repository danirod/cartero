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

use glib::{object::ObjectExt, subclass::types::ObjectSubclassIsExt};

use crate::entities::RequestPayload;

use super::{BasePayloadPane, BasePayloadPaneExt};

mod imp {
    use glib::subclass::{InitializingObject, Signal};
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate};
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use crate::entities::KeyValue;
    use crate::entities::KeyValueTable;
    use crate::objects::KeyValueItem;
    use crate::widgets::{BasePayloadPane, BasePayloadPaneImpl, KeyValuePane};

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::FormdataPayloadPane)]
    #[template(resource = "/es/danirod/Cartero/formdata_payload_pane.ui")]
    pub struct FormdataPayloadPane {
        #[template_child]
        data: TemplateChild<KeyValuePane>,

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
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("changed").build()])
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.data.assert_always_placeholder();

            let boundary = formdata::generate_boundary();
            let boundary = String::from_utf8_lossy(&boundary).to_string();
            self.boundary.set(boundary);

            self.data
                .connect_changed(glib::clone!(@weak self as pane => move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }));
        }
    }

    impl WidgetImpl for FormdataPayloadPane {}

    impl BasePayloadPaneImpl for FormdataPayloadPane {}

    impl FormdataPayloadPane {
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
    pub struct FormdataPayloadPane(ObjectSubclass<imp::FormdataPayloadPane>)
        @extends gtk::Widget, BasePayloadPane,
    @implements gtk::Accessible, gtk::Buildable;
}

impl FormdataPayloadPane {
    pub fn connect_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "changed",
            true,
            glib::closure_local!(|ref pane| {
                f(pane);
            }),
        )
    }
}

impl BasePayloadPaneExt for FormdataPayloadPane {
    fn payload(&self) -> RequestPayload {
        let imp = self.imp();
        let table = imp.get_table();
        RequestPayload::Multipart { params: table }
    }

    fn set_payload(&self, payload: &RequestPayload) {
        let imp = self.imp();
        if let RequestPayload::Multipart { params } = payload {
            imp.set_table(params);
        }
    }
}
