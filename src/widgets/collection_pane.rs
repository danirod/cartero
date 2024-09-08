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

use glib::{subclass::types::ObjectSubclassIsExt, Object};
use gtk::glib;

use crate::objects::Collection;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use glib::subclass::{InitializingObject, Signal};
    use glib::Properties;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate};

    use crate::objects::Collection;
    use crate::objects::KeyValueItem;
    use crate::widgets::KeyValuePane;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/es/danirod/Cartero/collection_pane.ui")]
    #[properties(wrapper_type = super::CollectionPane)]
    pub struct CollectionPane {
        #[template_child]
        collection_name: TemplateChild<gtk::Entry>,

        #[template_child]
        variables: TemplateChild<KeyValuePane>,

        #[property(get, set)]
        dirty: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollectionPane {
        const NAME: &'static str = "CarteroCollectionPane";
        type Type = super::CollectionPane;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CollectionPane {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("save-requested").build()])
        }
    }

    impl WidgetImpl for CollectionPane {}

    impl BoxImpl for CollectionPane {}

    #[gtk::template_callbacks]
    impl CollectionPane {
        #[template_callback]
        fn on_save(&self) {
            let obj = self.obj();

            obj.emit_by_name::<()>("save-requested", &[]);
        }

        pub(super) fn load_collection(&self, col: &Collection) {
            self.collection_name.set_text(&col.title());
            let variables: Vec<KeyValueItem> = col
                .variables_list()
                .iter()
                .map(|v| KeyValueItem::new_with_value(&v.header_name(), &v.header_value()))
                .collect();
            self.variables.set_entries(&variables);
        }

        pub(super) fn save_collection(&self, col: &Collection) {
            let name = self.collection_name.text();
            let variables = self.variables.get_entries();

            col.set_title(name);
            col.variables().remove_all();
            for variable in variables {
                col.add_variable(&variable);
            }
        }
    }
}

glib::wrapper! {
    pub struct CollectionPane(ObjectSubclass<imp::CollectionPane>)
        @extends gtk::Widget, gtk::Box;
}

impl Default for CollectionPane {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl CollectionPane {
    pub fn load_collection(&self, col: &Collection) {
        let imp = self.imp();
        imp.load_collection(col);
    }

    pub fn save_collection(&self, col: &Collection) {
        let imp = self.imp();
        imp.save_collection(col);
    }
}
