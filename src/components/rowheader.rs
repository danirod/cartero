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

use glib::subclass::InitializingObject;
use glib::GString;
use gtk4::subclass::prelude::*;
use gtk4::{prelude::*, CompositeTemplate};
use gtk4::{Box, Entry};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/es/danirod/Cartero/http_header_row.ui")]
pub struct RowHeaderImpl {
    #[template_child]
    pub entry_key: TemplateChild<Entry>,
    #[template_child]
    pub entry_value: TemplateChild<Entry>,
}

#[glib::object_subclass]
impl ObjectSubclass for RowHeaderImpl {
    const NAME: &'static str = "RowHeader";
    type Type = RowHeader;
    type ParentType = Box;
    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for RowHeaderImpl {
    fn constructed(&self) {
        self.parent_constructed();
    }
}
impl WidgetImpl for RowHeaderImpl {}
impl BoxImpl for RowHeaderImpl {}

glib::wrapper! {
    pub struct RowHeader(ObjectSubclass<RowHeaderImpl>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget,
                    gtk4::Actionable, gtk4::ActionBar, gtk4::ATContext;

}

impl RowHeader {
    pub fn new(key: &str, value: &str) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().entry_key.set_text(key);
        obj.imp().entry_value.set_text(value);
        obj
    }

    pub fn get_key(&self) -> GString {
        self.imp().entry_key.text()
    }

    pub fn get_value(&self) -> GString {
        self.imp().entry_value.text()
    }
}
