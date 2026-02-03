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
    use crate::settings::Settings;

    use super::*;

    use glib::subclass::InitializingObject;
    use gtk::{
        CompositeTemplate, FlowBox, FlowBoxChild, gio::SimpleActionGroup, pango::FontDescription,
    };
    use sourceview5::{StyleSchemeManager, StyleSchemePreview};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/page_appearance.ui")]
    pub struct Appearance {
        settings: Settings,

        #[template_child]
        option_theme: TemplateChild<adw::ComboRow>,
        #[template_child]
        option_use_system_font: TemplateChild<adw::SwitchRow>,
        #[template_child]
        option_custom_font: TemplateChild<gtk::FontDialogButton>,
        #[template_child]
        color_scheme_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        color_themes_light: TemplateChild<gtk::FlowBox>,
        #[template_child]
        color_themes_dark: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Appearance {
        const NAME: &'static str = "CarteroSettingsPageAppearance";
        type Type = super::Appearance;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Appearance {
        fn constructed(&self) {
            self.parent_constructed();

            self.settings
                .bind("use-system-font", &*self.option_use_system_font, "active")
                .build();

            self.settings
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

            self.settings
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

            let stack_page = if adw::StyleManager::default().is_dark() {
                "dark"
            } else {
                "light"
            };
            self.color_scheme_stack.set_visible_child_name(stack_page);
            let stack = self.color_scheme_stack.clone();
            adw::StyleManager::default().connect_dark_notify(move |style| {
                let stack_page = if style.is_dark() { "dark" } else { "light" };
                stack.set_visible_child_name(stack_page);
            });

            let action_group = SimpleActionGroup::new();
            let actions = ["color-scheme-light", "color-scheme-dark"];
            for action in actions {
                let action = self.settings.create_action(action);
                action_group.add_action(&action);
            }
            self.obj()
                .insert_action_group("widget", Some(&action_group));

            self.setup_color_themes();
            self.settings.connect_changed(
                Some("color-scheme-light"),
                glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _| {
                        imp.update_selected_theme();
                    }
                ),
            );
            self.settings.connect_changed(
                Some("color-scheme-dark"),
                glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _| {
                        imp.update_selected_theme();
                    }
                ),
            );
        }
    }

    impl WidgetImpl for Appearance {}

    impl PreferencesPageImpl for Appearance {}

    #[gtk::template_callbacks]
    impl Appearance {
        fn setup_color_themes(&self) {
            self.color_themes_light.remove_all();
            self.color_themes_dark.remove_all();

            let current_light = self.settings.get::<String>("color-scheme-light");
            let current_dark = self.settings.get::<String>("color-scheme-dark");

            let style_manager = StyleSchemeManager::default();

            let scheme_ids = style_manager.scheme_ids();
            let schemes = scheme_ids
                .iter()
                .filter_map(|scheme_id| style_manager.scheme(scheme_id.as_str()))
                .filter(|scheme| scheme.metadata("variant").is_some());

            for scheme in schemes {
                let variant = scheme
                    .metadata("variant")
                    .expect("Variant has dissappeared");
                let preview = StyleSchemePreview::builder().scheme(&scheme).build();
                preview.set_action_target(Some(scheme.id().to_string()));
                if variant.as_str() == "light" {
                    preview.set_selected(current_light == scheme.id().as_str());
                    preview.set_action_name(Some("widget.color-scheme-light"));
                    self.color_themes_light.append(&preview);
                } else {
                    preview.set_selected(current_dark == scheme.id().as_str());
                    preview.set_action_name(Some("widget.color-scheme-dark"));
                    self.color_themes_dark.append(&preview);
                }
            }

            self.update_selected_theme();
        }

        #[template_callback]
        fn flow_box_activate_child(_: &FlowBox, child: &FlowBoxChild) {
            if let Some(widget) = child.child().and_downcast::<StyleSchemePreview>() {
                widget.activate();
            }
        }

        fn update_selected_theme(&self) {
            let current_light = self.settings.get::<String>("color-scheme-light");
            let current_dark = self.settings.get::<String>("color-scheme-dark");

            let mut child = self.color_themes_light.first_child();
            while let Some(widget) = child.and_downcast::<gtk::FlowBoxChild>() {
                if let Some(preview) = widget.child().and_downcast::<StyleSchemePreview>() {
                    preview.set_selected(preview.scheme().id().as_str() == current_light);
                }
                child = widget.next_sibling();
            }

            let mut child = self.color_themes_dark.first_child();
            while let Some(widget) = child.and_downcast::<gtk::FlowBoxChild>() {
                if let Some(preview) = widget.child().and_downcast::<StyleSchemePreview>() {
                    preview.set_selected(preview.scheme().id().as_str() == current_dark);
                }
                child = widget.next_sibling();
            }
        }
    }
}
