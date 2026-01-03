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
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use crate::{settings::Settings, windows::settings::locale::LocaleRepr};

    use super::*;

    use glib::subclass::InitializingObject;
    use gtk::{ClosureExpression, CompositeTemplate};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/page_application.ui")]
    pub struct Application {
        settings: Settings,

        #[template_child]
        option_locale: TemplateChild<adw::ComboRow>,
        #[template_child]
        option_create_backups: TemplateChild<adw::SwitchRow>,
        #[template_child]
        group_updates: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        version_id: TemplateChild<adw::ActionRow>,
        #[template_child]
        locale_changed: TemplateChild<gtk::Revealer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "CarteroSettingsPageApplication";
        type Type = super::Application;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Application {
        fn constructed(&self) {
            self.parent_constructed();

            let locale_model = LocaleRepr::get_model();
            let expr: ClosureExpression =
                gtk::ClosureExpression::with_callback(gtk::Expression::NONE, |args| {
                    let repr = args[0].get::<LocaleRepr>().unwrap();
                    let iso = repr.iso();
                    let language = repr.name();
                    if !iso.is_empty() {
                        format!("{language} [{iso}]")
                    } else {
                        language
                    }
                });
            self.option_locale.set_expression(Some(&expr));
            self.option_locale.set_model(Some(&locale_model));

            self.settings
                .bind("locale", &*self.locale_changed, "reveal-child")
                .get_only()
                .mapping(|variant, _| {
                    let locale = variant.get::<String>().expect("Expected a string");
                    let current = std::env::var("LANGUAGE").unwrap_or(String::from(""));
                    Some((locale != current).to_value())
                })
                .build();

            self.settings
                .bind("locale", &*self.option_locale, "selected")
                .mapping(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    #[upgrade_or_panic]
                    move |variant, _| {
                        let locale = variant.get::<String>().expect("Expected a string");
                        let model = imp.option_locale.model().unwrap();
                        let result = model.iter::<LocaleRepr>().enumerate().find(|(_, obj)| {
                            match obj {
                                Ok(repr) => repr.iso() == locale,
                                Err(_) => false, // ???
                            }
                        });
                        result.map(|(idx, _)| (idx as u32).to_value())
                    }
                ))
                .set_mapping(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    #[upgrade_or_panic]
                    move |item, _| {
                        let index = item.get::<u32>().expect("What the heck");
                        let model = imp.option_locale.model().unwrap();
                        model
                            .item(index)
                            .and_downcast_ref::<LocaleRepr>()
                            .map(|repr| repr.iso().into())
                    }
                ))
                .build();

            self.version_id.set_subtitle(crate::config::VERSION);
            if cfg!(feature = "app_updater") {
                self.group_updates.set_visible(true);
            }

            self.settings
                .bind(
                    "create-backup-files",
                    &*self.option_create_backups,
                    "active",
                )
                .build();
        }
    }

    impl WidgetImpl for Application {}

    impl PreferencesPageImpl for Application {}
}
