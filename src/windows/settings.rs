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

use adw::prelude::AdwDialogExt;
use glib::{object::IsA, Object};
use gtk::gio;

mod imp {
    use adw::subclass::prelude::*;
    use glib::{object::ObjectExt, subclass::InitializingObject};
    use gtk::{
        pango::FontDescription, prelude::SettingsExtManual, template_callbacks, CompositeTemplate,
        TemplateChild,
    };

    use crate::app::CarteroApplication;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/settings_dialog.ui")]
    pub struct SettingsDialog {
        #[template_child]
        option_validate_tls: TemplateChild<adw::SwitchRow>,

        #[template_child]
        option_follow_redirects: TemplateChild<adw::ExpanderRow>,

        #[template_child]
        option_maximum_redirects: TemplateChild<adw::SpinRow>,

        #[template_child]
        option_timeout: TemplateChild<adw::SpinRow>,

        #[template_child]
        option_theme: TemplateChild<adw::ComboRow>,

        #[template_child]
        option_use_system_font: TemplateChild<adw::SwitchRow>,

        #[template_child]
        option_custom_font: TemplateChild<gtk::FontDialogButton>,

        #[template_child]
        option_create_backups: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsDialog {
        const NAME: &'static str = "CarteroSettingsDialog";
        type Type = super::SettingsDialog;
        type ParentType = adw::PreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
        }
    }

    impl WidgetImpl for SettingsDialog {}

    impl WindowImpl for SettingsDialog {}

    impl AdwDialogImpl for SettingsDialog {}

    impl PreferencesDialogImpl for SettingsDialog {}

    #[template_callbacks]
    impl SettingsDialog {
        fn init_settings(&self) {
            let app = CarteroApplication::default();
            let settings = app.settings();

            settings
                .bind("validate-tls", &*self.option_validate_tls, "active")
                .build();
            settings
                .bind(
                    "follow-redirects",
                    &*self.option_follow_redirects,
                    "enable-expansion",
                )
                .build();
            settings
                .bind(
                    "maximum-redirects",
                    &*self.option_maximum_redirects,
                    "value",
                )
                .build();
            settings
                .bind("request-timeout", &*self.option_timeout, "value")
                .build();

            settings
                .bind(
                    "create-backup-files",
                    &*self.option_create_backups,
                    "active",
                )
                .build();

            settings
                .bind("use-system-font", &*self.option_use_system_font, "active")
                .build();

            settings
                .bind("custom-font", &*self.option_custom_font, "font-desc")
                .mapping(|variant, _| {
                    let value = variant.get::<String>().expect("Expected a string");
                    Some(FontDescription::from_string(value.as_str()).into())
                })
                .set_mapping(|value, _| {
                    let value = value.get::<FontDescription>().expect("What?");
                    Some(value.to_string().into())
                })
                .build();

            self.option_use_system_font
                .bind_property("active", &*self.option_custom_font, "sensitive")
                .invert_boolean()
                .sync_create()
                .build();

            settings
                .bind("application-theme", &*self.option_theme, "selected")
                .mapping(|variant, _| {
                    let value = variant.get::<String>().expect("Expected a string");
                    let index: i32 = match value.as_str() {
                        "light" => 1,
                        "dark" => 2,
                        _ => 0,
                    };
                    Some(index.into())
                })
                .set_mapping(|value, _| {
                    let index = value.get::<u32>().expect("What the heck");
                    let setting = match index {
                        1 => "light",
                        2 => "dark",
                        _ => "system",
                    };
                    Some(setting.into())
                })
                .build();
        }
    }
}

glib::wrapper! {
    pub struct SettingsDialog(ObjectSubclass<imp::SettingsDialog>)
        @extends gtk::Widget, gtk::Window, adw::Dialog, adw::PreferencesDialog,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Root;
}

impl SettingsDialog {
    pub fn present_for_window(win: &impl IsA<gtk::Widget>) {
        let dialog: Self = Object::builder().build();
        dialog.present(Some(win));
    }
}
