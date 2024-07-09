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

use glib::object::ObjectExt;
use glib::property::PropertySet;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::{Binding, SignalHandlerId};
use gtk::gio::{self, PropertyAction, SimpleAction, SimpleActionGroup};
use gtk::glib::Object;
use gtk::prelude::WidgetExt;
use gtk::prelude::*;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use glib::subclass::{InitializingObject, Signal};
    use glib::{Binding, Properties, SignalHandlerId};
    use gtk::subclass::prelude::*;
    use gtk::Entry;
    use gtk::{prelude::*, CompositeTemplate};

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::KeyValueRow)]
    #[template(resource = "/es/danirod/Cartero/key_value_row.ui")]
    pub struct KeyValueRow {
        #[property(get, set)]
        active: RefCell<bool>,
        #[property(get, set)]
        secret: RefCell<bool>,
        #[property(get, set)]
        ignored: RefCell<bool>,

        #[property(get, set)]
        header_name: RefCell<String>,
        #[property(get, set)]
        header_value: RefCell<String>,

        #[template_child]
        pub entry_key: TemplateChild<Entry>,
        #[template_child]
        pub entry_value: TemplateChild<Entry>,

        pub bindings: RefCell<Vec<Binding>>,
        pub delete_signal: RefCell<Option<SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeyValueRow {
        const NAME: &'static str = "CarteroKeyValueRow";
        type Type = super::KeyValueRow;
        type ParentType = gtk::ListBoxRow;
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for KeyValueRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_actions();
            obj.setup_signals();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("delete").build()])
        }
    }

    impl WidgetImpl for KeyValueRow {}
    impl ListBoxRowImpl for KeyValueRow {}
}

glib::wrapper! {
    pub struct KeyValueRow(ObjectSubclass<imp::KeyValueRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Actionable, gtk::ActionBar, gtk::ATContext;

}

impl Default for KeyValueRow {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl KeyValueRow {
    pub(self) fn setup_signals(&self) {
        self.connect_active_notify(|row| {
            if row.active() {
                row.remove_css_class("inactive-header");
            } else {
                row.add_css_class("inactive-header");
            }
        });
        self.connect_ignored_notify(|row| {
            if row.ignored() {
                row.add_css_class("ignored-header");
            } else {
                row.remove_css_class("ignored-header");
            }
        });
    }

    pub(self) fn setup_actions(&self) {
        let ag = SimpleActionGroup::new();
        self.insert_action_group("row", Some(&ag));

        let toggle_secret = PropertyAction::new("toggle-secret", self, "secret");

        let delete = SimpleAction::new("delete", None);
        delete.connect_activate(glib::clone!(@weak self as widget => move |_, _| {
            widget.emit_by_name::<()>("delete", &[]);
        }));

        ag.add_action(&toggle_secret);
        ag.add_action(&delete);
    }

    pub fn add_binding(&self, binding: Binding) {
        let imp = self.imp();
        let mut bindings = imp.bindings.borrow_mut();
        bindings.push(binding);
    }

    pub fn reset_bindings(&self) {
        let imp = self.imp();
        let mut bindings = imp.bindings.borrow_mut();
        while bindings.len() > 0 {
            if let Some(binding) = bindings.pop() {
                binding.unbind();
            }
        }
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
