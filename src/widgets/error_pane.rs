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
use std::error::Error;
use url::ParseError;

use crate::{
    client::RequestError,
    error::{RequestBuildError, RequestPreconditionError},
};

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

    pub fn set_request_build_error(&self, error: RequestBuildError) {
        self.set_icon("dialog-warning-symbolic");
        self.set_title(gettext("The request data is not valid"));
        self.set_subtitle(error.to_string());

        self.set_extra("");
        if let Some(error) = error.source() {
            if let Some(pe) = error.downcast_ref::<ParseError>() {
                self.set_extra(i18n_url_parse_error(pe).unwrap_or_default());
            }
        }
    }
}

impl Default for ErrorPane {
    fn default() -> Self {
        glib::Object::new()
    }
}

fn i18n_url_parse_error(pe: &ParseError) -> Option<String> {
    match pe {
        ParseError::EmptyHost
        | ParseError::IdnaError
        | ParseError::InvalidDomainCharacter
        | ParseError::SetHostOnCannotBeABaseUrl => {
            Some(gettext("The host is missing or malformed"))
        }
        ParseError::InvalidPort => Some(gettext("The specified port number is not valid")),
        ParseError::InvalidIpv4Address | ParseError::InvalidIpv6Address => {
            Some(gettext("The IP specified is not valid"))
        }
        ParseError::RelativeUrlWithCannotBeABaseBase | ParseError::RelativeUrlWithoutBase => {
            Some(gettext("Relative URL cannot be solved"))
        }
        ParseError::Overflow => Some(gettext("The specified URL is too long for this program")),
        _ => None,
    }
}
