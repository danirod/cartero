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
use gettextrs::gettext;
use glib::subclass::InitializingObject;
use glib::Properties;
use gtk::CompositeTemplate;
use std::cell::RefCell;

use crate::{client::RequestError, error::RequestPreconditionError};

mod imp {
    use super::*;

    #[derive(Default, Properties, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/error_pane.ui")]
    #[properties(wrapper_type = super::ErrorPane)]
    pub struct ErrorPane {
        #[property(get, set)]
        icon: RefCell<String>,

        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set)]
        subtitle: RefCell<String>,

        #[property(get, set)]
        extra: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ErrorPane {
        const NAME: &'static str = "CarteroErrorPane";
        type Type = super::ErrorPane;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ErrorPane {}

    impl WidgetImpl for ErrorPane {}

    impl BinImpl for ErrorPane {}

    #[gtk::template_callbacks]
    impl ErrorPane {
        #[template_callback]
        fn extra_visible(&self) -> bool {
            let extra = self.extra.borrow();
            !extra.is_empty()
        }
    }
}

glib::wrapper! {
    pub struct ErrorPane(ObjectSubclass<imp::ErrorPane>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Buildable;
}

impl ErrorPane {
    pub fn set_precondition_error(&self, error: RequestPreconditionError) {
        self.set_icon("dialog-warning-symbolic");
        self.set_title(gettext("The request data is not valid"));
        self.set_subtitle(error.to_string());
    }

    pub fn set_request_error(&self, error: RequestError) {
        self.set_icon("network-error-symbolic");
        self.set_title(gettext("The HTTP request failed"));
        self.set_subtitle(error.to_string());
    }
}

impl Default for ErrorPane {
    fn default() -> Self {
        glib::Object::new()
    }
}
