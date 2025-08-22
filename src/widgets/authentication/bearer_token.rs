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

    use super::*;
    use cartero_objects::RequestAuthenticationBearer;
    use glib::{subclass::InitializingObject, BindingGroup, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::BearerToken)]
    #[template(resource = "/es/danirod/Cartero/bearer_token_pane.ui")]
    pub struct BearerToken {
        #[property(get, set)]
        bearer_token: RefCell<RequestAuthenticationBearer>,
        #[property(get)]
        binding_group: RefCell<BindingGroup>,

        #[template_child]
        token: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BearerToken {
        const NAME: &'static str = "CarteroBearerTokenPane";
        type Type = super::BearerToken;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for BearerToken {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.binding_group()
                .bind("token", &*self.token, "text")
                .bidirectional()
                .sync_create()
                .build();
            obj.bind_property("bearer-token", &obj.binding_group(), "source")
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for BearerToken {}

    impl BinImpl for BearerToken {}

    impl BearerToken {}
}

glib::wrapper! {
    pub struct BearerToken(ObjectSubclass<imp::BearerToken>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
