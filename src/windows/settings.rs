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

mod locale;

use adw::prelude::AdwDialogExt;
use glib::{
    object::{Cast, IsA, ObjectExt},
    Object,
};
use gtk::{
    gio::{self},
    prelude::WidgetExt,
};

mod imp {
    use std::sync::OnceLock;

    use adw::prelude::{ActionRowExt, ComboRowExt, WidgetExt};
    use adw::subclass::prelude::*;
    use glib::object::CastNone;
    use glib::subclass::Signal;
    use glib::value::ToValue;
    use glib::{object::ObjectExt, subclass::InitializingObject};
    use gtk::ClosureExpression;
    use gtk::{
        pango::FontDescription, prelude::SettingsExtManual, template_callbacks, CompositeTemplate,
        TemplateChild,
    };
    use sourceview5::prelude::{ListModelExt, ListModelExtManual};

    use crate::app::CarteroApplication;

    use super::locale::LocaleRepr;

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
        option_locale: TemplateChild<adw::ComboRow>,

        #[template_child]
        option_theme: TemplateChild<adw::ComboRow>,

        #[template_child]
        option_use_system_font: TemplateChild<adw::SwitchRow>,

        #[template_child]
        option_custom_font: TemplateChild<gtk::FontDialogButton>,

        #[template_child]
        option_create_backups: TemplateChild<adw::SwitchRow>,

        #[template_child]
        group_updates: TemplateChild<adw::PreferencesGroup>,

        #[template_child]
        version_id: TemplateChild<adw::ActionRow>,

        #[template_child]
        locale_changed: TemplateChild<adw::Banner>,
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

            // Init locale list before loading settings.
            let locale_model = LocaleRepr::get_model();
            self.option_locale.set_model(Some(&locale_model));
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

            self.init_settings();
            self.init_locale_banner();

            self.version_id.set_subtitle(crate::config::VERSION);
            if cfg!(feature = "app_updater") {
                self.group_updates.set_visible(true);
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("check-updates").build()])
        }
    }

    impl WidgetImpl for SettingsDialog {}

    impl WindowImpl for SettingsDialog {}

    impl AdwDialogImpl for SettingsDialog {}

    impl PreferencesDialogImpl for SettingsDialog {}

    #[template_callbacks]
    impl SettingsDialog {
        fn init_locale_banner(&self) {
            let app = CarteroApplication::default();
            let settings = app.settings();

            settings
                .bind("locale", &*self.locale_changed, "revealed")
                .get_only()
                .mapping(|variant, _| {
                    let locale = variant.get::<String>().expect("Expected a string");
                    let current = std::env::var("LANGUAGE").unwrap_or(String::from(""));
                    Some((locale != current).to_value())
                })
                .build();
        }

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

            settings
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
        }

        #[template_callback]
        fn on_check_updates(&self) {
            let obj = self.obj();
            obj.emit_by_name::<()>("check-updates", &[]);
        }
    }
}

glib::wrapper! {
    pub struct SettingsDialog(ObjectSubclass<imp::SettingsDialog>)
        @extends gtk::Widget, gtk::Window, adw::Dialog, adw::PreferencesDialog,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Root;
}

impl SettingsDialog {
    pub fn connect_check_updates<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "check-updates",
            true,
            glib::closure_local!(|dialog| {
                f(dialog);
            }),
        )
    }

    pub fn present_for_window(win: &impl IsA<gtk::Widget>) {
        let dialog: Self = Object::builder().build();
        if let Some(window) = win.as_ref().downcast_ref::<gtk::Window>() {
            dialog.connect_closed(glib::clone!(
                #[weak]
                window,
                move |_| {
                    adw::prelude::GtkWindowExt::present(&window);
                }
            ));

            if cfg!(feature = "app_updater") {
                dialog.connect_check_updates(glib::clone!(
                    #[weak]
                    window,
                    move |_| {
                        let _ = window.activate_action("win.check-updates", None);
                    }
                ));
            }
        }

        dialog.present(Some(win));
    }
}
