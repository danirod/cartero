// Copyright 2024-2026 the Cartero authors
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

use adw::prelude::*;
use adw::subclass::prelude::*;

glib::wrapper! {
    pub struct CodeEditor(ObjectSubclass<imp::CodeEditor>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CodeEditor {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use crate::settings::Settings;

    use super::*;

    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, gio::SimpleActionGroup};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/page_code_editor.ui")]
    pub struct CodeEditor {
        settings: Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CodeEditor {
        const NAME: &'static str = "CarteroSettingsPageCodeEditor";
        type Type = super::CodeEditor;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CodeEditor {
        fn constructed(&self) {
            self.parent_constructed();

            let action_group = SimpleActionGroup::new();

            let actions = [
                "auto-indent",
                "body-wrap",
                "indent-style",
                "show-line-numbers",
                "tab-width",
            ];
            for action in actions {
                let action = self.settings.create_action(action);
                action_group.add_action(&action);
            }
            self.obj()
                .insert_action_group("widget", Some(&action_group));
        }
    }

    impl WidgetImpl for CodeEditor {}

    impl PreferencesPageImpl for CodeEditor {}
}
