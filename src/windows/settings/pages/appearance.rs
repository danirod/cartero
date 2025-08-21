// Copyright 2024-2025 the Cartero authors
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
    pub struct Appearance(ObjectSubclass<imp::Appearance>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Appearance {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use crate::config::BASE_ID;

    use super::*;

    use glib::subclass::InitializingObject;
    use gtk::{gio::Settings, pango::FontDescription, CompositeTemplate};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/page_appearance.ui")]
    pub struct Appearance {
        #[template_child]
        option_theme: TemplateChild<adw::ComboRow>,
        #[template_child]
        option_use_system_font: TemplateChild<adw::SwitchRow>,
        #[template_child]
        option_custom_font: TemplateChild<gtk::FontDialogButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Appearance {
        const NAME: &'static str = "CarteroSettingsPageAppearance";
        type Type = super::Appearance;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Appearance {
        fn constructed(&self) {
            self.parent_constructed();

            let settings = Settings::new(BASE_ID);

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

    impl WidgetImpl for Appearance {}

    impl PreferencesPageImpl for Appearance {}
}
