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
    pub struct HttpClient(ObjectSubclass<imp::HttpClient>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HttpClient {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use crate::config::BASE_ID;

    use super::*;

    use glib::subclass::InitializingObject;
    use gtk::{gio::Settings, CompositeTemplate};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/settings/page_http_client.ui")]
    pub struct HttpClient {
        #[template_child]
        option_validate_tls: TemplateChild<adw::SwitchRow>,
        #[template_child]
        option_follow_redirects: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        option_maximum_redirects: TemplateChild<adw::SpinRow>,
        #[template_child]
        option_timeout: TemplateChild<adw::SpinRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HttpClient {
        const NAME: &'static str = "CarteroSettingsPageHttpClient";
        type Type = super::HttpClient;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HttpClient {
        fn constructed(&self) {
            self.parent_constructed();
            let settings = Settings::new(BASE_ID);

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
        }
    }

    impl WidgetImpl for HttpClient {}

    impl PreferencesPageImpl for HttpClient {}
}
