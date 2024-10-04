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
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/save_dialog.ui")]
    pub struct SaveDialog;

    #[glib::object_subclass]
    impl ObjectSubclass for SaveDialog {
        const NAME: &'static str = "CarteroSaveDialog";
        type Type = super::SaveDialog;
        type ParentType = adw::AlertDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SaveDialog {}

    impl WidgetImpl for SaveDialog {}

    impl AdwDialogImpl for SaveDialog {}

    impl AdwAlertDialogImpl for SaveDialog {}
}

glib::wrapper! {
    pub struct SaveDialog(ObjectSubclass<imp::SaveDialog>)
        @extends gtk::Widget, adw::Dialog, adw::AlertDialog,
        @implements gtk::Accessible, gtk::Buildable;
}

impl Default for SaveDialog {
    fn default() -> Self {
        Object::builder().build()
    }
}
