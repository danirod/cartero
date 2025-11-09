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

mod imp {
    use std::cell::RefCell;

    use crate::app::CarteroApplication;

    use super::*;
    use cartero_objects::RequestAuthenticationBasic;
    use glib::{subclass::InitializingObject, BindingGroup, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::BasicAuth)]
    #[template(resource = "/es/danirod/Cartero/basic_auth_pane.ui")]
    pub struct BasicAuth {
        #[property(get, set)]
        basic_auth: RefCell<RequestAuthenticationBasic>,
        #[property(get)]
        binding_group: RefCell<BindingGroup>,

        #[template_child]
        username: TemplateChild<adw::EntryRow>,
        #[template_child]
        password: TemplateChild<adw::PasswordEntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BasicAuth {
        const NAME: &'static str = "CarteroBasicAuthPane";
        type Type = super::BasicAuth;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for BasicAuth {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.binding_group()
                .bind("username", &*self.username, "text")
                .bidirectional()
                .sync_create()
                .build();
            obj.binding_group()
                .bind("password", &*self.password, "text")
                .bidirectional()
                .sync_create()
                .build();
            obj.bind_property("basic-auth", &obj.binding_group(), "source")
                .sync_create()
                .build();

            if let Some(delegate) = self.password.delegate().and_downcast::<gtk::Text>() {
                let settings = CarteroApplication::ui_settings();
                settings
                    .bind("conceal-basic-auth-password", &delegate, "visibility")
                    .invert_boolean()
                    .build();
            }
        }
    }

    impl WidgetImpl for BasicAuth {}

    impl BinImpl for BasicAuth {}

    impl BasicAuth {}
}

glib::wrapper! {
    pub struct BasicAuth(ObjectSubclass<imp::BasicAuth>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
