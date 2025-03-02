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
use gtk::prelude::*;

mod imp {
    use std::cell::RefCell;

    use adw::subclass::bin::BinImpl;
    use glib::subclass::InitializingObject;
    use glib::Properties;
    use gtk::gdk::Rectangle;
    use gtk::subclass::prelude::*;
    use gtk::{prelude::*, CompositeTemplate, Inscription, PopoverMenu};

    use crate::win::CarteroWindow;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SidebarRow)]
    #[template(resource = "/es/danirod/Cartero/sidebar_row.ui")]
    pub struct SidebarRow {
        #[template_child]
        pub(super) inscription: TemplateChild<Inscription>,

        #[template_child]
        context_menu: TemplateChild<PopoverMenu>,

        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set)]
        path: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarRow {
        const NAME: &'static str = "CarteroSidebarRow";
        type Type = super::SidebarRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("row.close-collection", None, |row, _, _| {
                if let Some(window) = row.root().and_downcast::<CarteroWindow>() {
                    let row_imp = row.imp();
                    row_imp.context_menu.popdown();
                    row_imp.context_menu.unparent();
                    window.close_collection(&row.path());
                }
            });
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SidebarRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.deactivate_actions();
        }
    }

    impl WidgetImpl for SidebarRow {}

    impl BinImpl for SidebarRow {}

    #[gtk::template_callbacks]
    impl SidebarRow {
        #[template_callback]
        fn on_right_click(&self, _n: i32, x: f64, y: f64) {
            let obj = &*self.obj();

            let rect = Rectangle::new(x as i32, y as i32, 0, 0);
            self.context_menu.set_pointing_to(Some(&rect));
            self.context_menu.unparent();
            self.context_menu.set_parent(obj);
            self.context_menu.popup();
        }
    }
}

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for SidebarRow {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarRow {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn deactivate_actions(&self) {
        self.action_set_enabled("row.close-collection", false);
    }

    pub fn activate_collection_actions(&self) {
        self.action_set_enabled("row.close-collection", true);
    }
}
