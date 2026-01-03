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

mod imp {
    use std::cell::RefCell;

    use super::*;
    use glib::subclass::InitializingObject;
    use gtk::{
        gio::{SimpleAction, SimpleActionGroup},
        CompositeTemplate,
    };

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/welcome/welcome_pane.ui")]
    pub struct WelcomePane {
        #[template_child]
        rights: TemplateChild<adw::Dialog>,

        carousel_position: RefCell<u32>,
        #[template_child]
        carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        previous: TemplateChild<gtk::Button>,
        #[template_child]
        next: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WelcomePane {
        const NAME: &'static str = "CarteroWelcomePane";
        type Type = super::WelcomePane;
        type ParentType = adw::BreakpointBin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WelcomePane {
        fn constructed(&self) {
            self.parent_constructed();

            let action_rights = SimpleAction::new("license-conditions", None);
            action_rights.connect_activate(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.reset_license_tour();
                }
            ));

            let action_group = SimpleActionGroup::new();
            action_group.add_action(&action_rights);
            self.obj()
                .insert_action_group("widget", Some(&action_group));
        }
    }

    impl WidgetImpl for WelcomePane {}

    impl BreakpointBinImpl for WelcomePane {}

    #[gtk::template_callbacks]
    impl WelcomePane {
        fn reset_license_tour(&self) {
            self.carousel_position.replace(0);
            let page = self.carousel.nth_page(0);
            self.carousel.scroll_to(&page, false);
            self.update_buttons();
            self.rights.present(Some(&*self.obj()));
        }

        #[template_callback]
        fn next_page(&self) {
            let pos = *self.carousel_position.borrow();
            if pos <= self.carousel.n_pages() - 2 {
                self.switch_page(pos + 1);
            }
        }

        #[template_callback]
        fn previous_page(&self) {
            let pos = *self.carousel_position.borrow();
            if pos > 0 {
                self.switch_page(pos - 1);
            }
        }

        fn switch_page(&self, pos: u32) {
            let page = self.carousel.nth_page(pos);
            self.carousel.scroll_to(&page, true);
            self.carousel_position.replace(pos);
            self.update_buttons();
        }

        fn update_buttons(&self) {
            let pos = *self.carousel_position.borrow();
            let is_first = pos == 0;
            let is_last = pos == self.carousel.n_pages() - 1;
            self.next.set_sensitive(!is_last);
            self.previous.set_sensitive(!is_first);

            if is_first {
                self.next.add_css_class("suggested-action");
            } else {
                self.next.remove_css_class("suggested-action");
            }
        }
    }
}

glib::wrapper! {
    pub struct WelcomePane(ObjectSubclass<imp::WelcomePane>)
        @extends gtk::Widget, adw::BreakpointBin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for WelcomePane {
    fn default() -> Self {
        glib::Object::new()
    }
}
