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
use gtk::gio;

glib::wrapper! {
    pub struct Shell(ObjectSubclass<imp::Shell>)
        @extends gtk::Widget, adw::BreakpointBin,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Shell {
    fn default() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use std::cell::OnceCell;

    use crate::app::CarteroApplication;
    use crate::widgets::endpoint::EndpointPane;
    use crate::widgets::shell::{BasePane, CommonShell};
    use crate::widgets::standalone::closures::install_base_pane_page_closures;

    use super::*;
    use glib::subclass::InitializingObject;
    use gtk::gdk;
    use gtk::{
        gio::{ActionEntry, SimpleActionGroup},
        CompositeTemplate,
    };

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/es/danirod/Cartero/standalone/shell.ui")]
    pub struct Shell {
        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,

        #[template_child]
        tabview: TemplateChild<adw::TabView>,

        #[template_child]
        window_title: TemplateChild<adw::WindowTitle>,
        window_title_binding_group: OnceCell<glib::BindingGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Shell {
        const NAME: &'static str = "CarteroStandaloneShell";
        type Type = super::Shell;
        type ParentType = adw::BreakpointBin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            let primary = if cfg!(target_os = "macos") {
                gdk::ModifierType::META_MASK
            } else {
                gdk::ModifierType::CONTROL_MASK
            };

            // The new tab shortcut is kinda special
            klass.add_binding_action(gdk::Key::T, primary, "win.new-endpoint");
            klass.add_binding_action(gdk::Key::O, primary, "win.open-tab");
            klass.add_binding_action(
                gdk::Key::T,
                primary | gdk::ModifierType::SHIFT_MASK,
                "app.new-window",
            );
            klass.add_binding_action(gdk::Key::S, primary, "win.save-tab");
            klass.add_binding_action(
                gdk::Key::S,
                primary | gdk::ModifierType::SHIFT_MASK,
                "win.save-tab-as",
            );
            klass.add_binding_action(gdk::Key::W, primary, "win.close-tab");
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Shell {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_actions();
            self.connect_to_window();
            self.init_window_title();
            self.obj()
                .connect_root_notify(|win| win.imp().connect_to_window());
        }
    }

    impl WidgetImpl for Shell {}

    impl BreakpointBinImpl for Shell {}

    impl CommonShell for Shell {}

    #[gtk::template_callbacks]
    impl Shell {
        #[template_callback]
        fn on_page_change(&self) {
            let selected_page = self.tabview.selected_page();
            if let Some(bg) = self.window_title_binding_group.get() {
                bg.set_source(selected_page.as_ref());
            }

            let window_title = match &selected_page {
                Some(page) => page
                    .property_expression("title")
                    .chain_closure::<String>(glib::closure!(move |_: glib::Object, title: &str| {
                        format!("{title} — Cartero")
                    }))
                    .upcast(),
                None => gtk::ConstantExpression::new("Cartero").upcast(),
            };
            if let Some(window) = self.get_application_window() {
                window_title.bind(&window, "title", selected_page.as_ref());

                if selected_page.is_some() {
                    self.header_bar.set_title_widget(Some(&*self.window_title));
                } else {
                    self.header_bar.set_title_widget(gtk::Widget::NONE);
                }
            }
        }

        #[template_callback]
        fn on_window_create_request() -> Option<adw::TabView> {
            let shell = CarteroApplication::get().new_window();
            Some(shell.imp().tabview.clone())
        }

        /// Create a binding between the current tab title and tooltip and
        /// the window title. The source will be the currently visible page,
        /// if one is even visible at the moment.
        fn init_window_title(&self) {
            let binding_group = glib::BindingGroup::new();
            binding_group
                .bind("title", &*self.window_title, "title")
                .sync_create()
                .build();
            binding_group
                .bind("tooltip", &*self.window_title, "subtitle")
                .sync_create()
                .build();

            // Persist it or it will be dropped and removed when this function ends.
            binding_group.set_source(self.tabview.selected_page().as_ref());
            self.window_title_binding_group
                .set(binding_group)
                .expect("Cannot assign window_title_binding_group");
        }

        /// Callback invoked whenever the shell gets connected to a window.
        /// Supposedly, this should only be called at most twice, because you
        /// don't usually swap the root for the shell in runtime.
        fn connect_to_window(&self) {
            /* Disable the header bar controls unless we are in CSD mode. */
            let csd = self.is_client_side();
            self.header_bar.set_show_end_title_buttons(csd);
            self.header_bar.set_show_start_title_buttons(csd);
            self.header_bar.set_show_title(csd);

            /* Use native controls if the SDK 49 is supported. */
            if adw::major_version() == 1 && adw::minor_version() >= 8 {
                self.header_bar.set_property("use-native-controls", true);
            }

            /* Invoke tab-page <-> window related hooks. */
            self.on_page_change();
        }

        fn init_actions(&self) {
            let new = ActionEntry::builder("new-endpoint")
                .activate(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _, _| imp.action_new_endpoint()
                ))
                .build();

            let open = ActionEntry::builder("open-tab")
                .activate(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _, _| imp.action_open_tab()
                ))
                .build();

            let save = ActionEntry::builder("save-tab")
                .activate(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _, _| imp.action_save_tab()
                ))
                .build();

            let save_as = ActionEntry::builder("save-tab-as")
                .activate(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _, _| imp.action_save_tab_as()
                ))
                .build();

            let close = ActionEntry::builder("close-tab")
                .activate(glib::clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_, _, _| imp.action_close_tab()
                ))
                .build();

            let group = SimpleActionGroup::new();
            group.add_action_entries([new, open, save, save_as, close]);
            self.obj().insert_action_group("win", Some(&group));

            // And then, bind the availability of some of these actions to the
            // availability of a pane. You might be noticing I am using
            // lookup_action() again instead of just reusing the variables
            // from above... but for some reason ActionEntry objects are just
            // structs that lack a "enabled" property, so I have to look them
            // up again.
            let has_page = self
                .tabview
                .property_expression("selected-page")
                .chain_closure::<bool>(glib::closure!(
                    move |_: glib::Object, page: Option<&adw::TabPage>| page.is_some()
                ));
            let tab_dependent_actions = ["save-tab", "save-tab-as", "close-tab"];
            for tab in tab_dependent_actions {
                if let Some(action) = group.lookup_action(tab) {
                    has_page.bind(&action, "enabled", Some(&*self.tabview));
                }
            }
        }

        fn action_new_endpoint(&self) {
            let endpoint_pane = EndpointPane::default();
            let base_pane = endpoint_pane.upcast::<BasePane>();
            let page = self.tabview.add_page(&base_pane, None);
            install_base_pane_page_closures(&page, &base_pane);
        }

        fn action_open_tab(&self) {
            println!("TODO: implement open");
        }

        fn action_save_tab(&self) {
            println!("TODO: implement save");
        }

        fn action_save_tab_as(&self) {
            println!("TODO: implement save as");
        }

        fn action_close_tab(&self) {
            if let Some(current_page) = self.tabview.selected_page() {
                self.tabview.close_page(&current_page);
            }
        }
    }
}
