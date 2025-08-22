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
    pub struct Proxy(ObjectSubclass<imp::Proxy>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Proxy {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use crate::config::BASE_ID;

    use super::*;

    use gettextrs::gettext;
    use glib::subclass::InitializingObject;
    use gtk::{
        gio::{Settings, SimpleActionGroup},
        CompositeTemplate,
    };

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/page_proxy.ui")]
    pub struct Proxy {
        #[template_child]
        option_proxy_http: TemplateChild<adw::EntryRow>,
        #[template_child]
        option_proxy_https: TemplateChild<adw::EntryRow>,
        #[template_child]
        option_std_http: TemplateChild<adw::ActionRow>,
        #[template_child]
        option_std_https: TemplateChild<adw::ActionRow>,
        #[template_child]
        option_std_no_proxy: TemplateChild<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Proxy {
        const NAME: &'static str = "CarteroSettingsPageProxy";
        type Type = super::Proxy;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Proxy {
        fn constructed(&self) {
            self.parent_constructed();
            let settings = Settings::new(BASE_ID);

            let action_use_std = settings.create_action("proxy-use-env");
            let action_group = SimpleActionGroup::new();
            action_group.add_action(&action_use_std);
            self.obj()
                .insert_action_group("widget", Some(&action_group));

            settings
                .bind("proxy-http", &*self.option_proxy_http, "text")
                .build();
            settings
                .bind("proxy-https", &*self.option_proxy_https, "text")
                .build();

            self.option_std_http.set_subtitle(&env_var("http_proxy"));
            self.option_std_https.set_subtitle(&env_var("https_proxy"));
            self.option_std_no_proxy.set_subtitle(&env_var("no_proxy"));
        }
    }

    impl WidgetImpl for Proxy {}

    impl PreferencesPageImpl for Proxy {}

    fn env_var(key: &str) -> String {
        let var = std::env::var(key).unwrap_or_default();
        if var.is_empty() {
            gettext("(none)")
        } else {
            var
        }
    }
}
