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

use glib::property::PropertySet;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::{Binding, SignalHandlerId};
use gtk::glib;
use gtk::glib::Object;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use glib::subclass::{InitializingObject, Signal};
    use glib::{Properties, SignalHandlerId};
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate};
    use gtk::{Box, Entry};

    use super::HeaderRowBindings;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::RequestHeaderRow)]
    #[template(resource = "/es/danirod/Cartero/request_header_row.ui")]
    pub struct RequestHeaderRow {
        #[property(get, set)]
        active: RefCell<bool>,
        #[property(get, set)]
        header_name: RefCell<String>,
        #[property(get, set)]
        header_value: RefCell<String>,
        #[template_child]
        pub entry_key: TemplateChild<Entry>,
        #[template_child]
        pub entry_value: TemplateChild<Entry>,

        pub bindings: RefCell<Option<HeaderRowBindings>>,

        pub delete_signal: RefCell<Option<SignalHandlerId>>,
    }

    #[gtk::template_callbacks]
    impl RequestHeaderRow {
        #[template_callback]
        fn on_delete_request(&self) {
            let obj = self.obj();
            obj.emit_by_name::<()>("delete", &[]);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RequestHeaderRow {
        const NAME: &'static str = "CarteroRequestHeaderRow";
        type Type = super::RequestHeaderRow;
        type ParentType = Box;
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for RequestHeaderRow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("delete").build()])
        }
    }

    impl WidgetImpl for RequestHeaderRow {}
    impl BoxImpl for RequestHeaderRow {}
}

glib::wrapper! {
    pub struct RequestHeaderRow(ObjectSubclass<imp::RequestHeaderRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Actionable, gtk::ActionBar, gtk::ATContext;

}

impl Default for RequestHeaderRow {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl RequestHeaderRow {
    pub fn set_bindings(&self, header_name: Binding, header_value: Binding, active: Binding) {
        let imp = self.imp();

        let binds = HeaderRowBindings::new(header_name, header_value, active);
        imp.bindings.set(Some(binds));
    }

    pub fn reset_bindings(&self) {
        let imp = self.imp();

        {
            let bindings = imp.bindings.borrow_mut();
            if let Some(binds) = &*bindings {
                binds.unbind();
            }
        }

        imp.bindings.set(None);
    }

    pub fn set_delete_closure(&self, hnd: SignalHandlerId) {
        let imp = self.imp();
        imp.delete_signal.set(Some(hnd));
    }

    pub fn delete_closure(&self) -> Option<SignalHandlerId> {
        let imp = self.imp();
        imp.delete_signal.take()
    }
}

/// Represents the bindings used by the HeaderRow component, which are required to
/// persist so that I can unbind them later when the component is being cleaned
/// to be reused with a different header later.
pub struct HeaderRowBindings {
    header_name: Binding,
    header_value: Binding,
    active: Binding,
}

impl HeaderRowBindings {
    pub fn new(header_name: Binding, header_value: Binding, active: Binding) -> Self {
        Self {
            header_name,
            header_value,
            active,
        }
    }

    pub fn unbind(&self) {
        self.header_name.unbind();
        self.header_value.unbind();
        self.active.unbind();
    }
}
