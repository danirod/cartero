// Copyright 2024 the Cartero authors
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

use crate::app::CarteroApplication;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::Object;
use gtk::{
    gio::{self, Settings},
    glib,
};

use gtk::prelude::ActionMapExt;
use gtk::prelude::SettingsExt;

mod imp {
    use adw::subclass::application_window::AdwApplicationWindowImpl;
    use gtk::gio::ActionEntry;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::error::CarteroError;
    use crate::widgets::*;
    use glib::subclass::InitializingObject;
    use gtk::{
        subclass::{
            application_window::ApplicationWindowImpl, widget::WidgetImpl, window::WindowImpl,
        },
        CompositeTemplate, TemplateChild,
    };

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/main_window.ui")]
    pub struct CarteroWindow {
        #[template_child]
        pub pane: TemplateChild<EndpointPane>,
    }

    #[gtk::template_callbacks]
    impl CarteroWindow {
        /// Returns the pane currently visible in the window.
        ///
        /// This method will make more sense in the future once multiple panes can be visible in tabs.
        pub fn current_pane(&self) -> &EndpointPane {
            &self.pane
        }

        async fn trigger_open(&self) -> Result<(), CarteroError> {
            // In order to place the modal, we need a reference to the public type.
            let obj = self.obj();
            let path = crate::widgets::open_file(&obj).await?;

            if let Some(path) = path {
                let contents = crate::file::read_file(&path)?;
                let endpoint = crate::file::parse_toml(&contents)?;
                self.current_pane().assign_endpoint(endpoint);
            }
            Ok(())
        }

        async fn trigger_save(&self) -> Result<(), CarteroError> {
            // In order to place the modal, we need a reference to the public type.
            let obj = self.obj();
            let path = crate::widgets::save_file(&obj).await?;

            if let Some(path) = path {
                let endpoint = self.current_pane().extract_endpoint()?;
                let serialized_payload = crate::file::store_toml(endpoint)?;
                crate::file::write_file(&path, &serialized_payload)?;
            }
            Ok(())
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroWindow {
        const NAME: &'static str = "CarteroWindow";
        type Type = super::CarteroWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            KeyValueRow::static_type();
            KeyValuePane::static_type();
            EndpointPane::static_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CarteroWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let action_request = ActionEntry::builder("request")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    if let Err(e) = window.current_pane().perform_request() {
                        let error_msg = format!("{}", e);
                        window.current_pane().show_revealer(&error_msg)
                    }
                }))
                .build();
            let action_open = ActionEntry::builder("open")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_open().await {
                            let error_msg = format!("{}", e);
                            window.current_pane().show_revealer(&error_msg);
                        }
                    }));
                }))
                .build();
            let action_save = ActionEntry::builder("save")
                .activate(glib::clone!(@weak self as window => move |_, _, _| {
                    glib::spawn_future_local(glib::clone!(@weak window => async move {
                        if let Err(e) = window.trigger_save().await {
                            let error_msg = format!("{}", e);
                            window.current_pane().show_revealer(&error_msg);
                        }
                    }));
                }))
                .build();

            let obj = self.obj();
            obj.add_action_entries([action_request, action_open, action_save]);
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {}

    impl ApplicationWindowImpl for CarteroWindow {}

    impl AdwApplicationWindowImpl for CarteroWindow {}
}

glib::wrapper! {
    pub struct CarteroWindow(ObjectSubclass<imp::CarteroWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CarteroWindow {
    pub fn new(app: &CarteroApplication) -> Self {
        Object::builder().property("application", Some(app)).build()
    }

    pub fn assign_settings(&self, settings: &Settings) {
        let imp = &self.imp();

        let wrap = settings.create_action("body-wrap");
        self.add_action(&wrap);

        imp.current_pane().bind_settings(settings);
    }

    pub fn show_revealer(&self, str: &str) {
        let imp = &self.imp();
        imp.current_pane().show_revealer(str);
    }

    pub fn hide_revealer(&self) {
        let imp = &self.imp();
        imp.current_pane().hide_revealer();
    }
}
