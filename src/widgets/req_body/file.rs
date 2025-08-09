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
    use std::{
        cell::{OnceCell, RefCell},
        path::PathBuf,
    };

    use crate::{
        widgets::{dialogs::glib_file_dialog_error, endpoint::EndpointPane, pick_file},
        win::CarteroWindow,
    };

    use super::*;
    use cartero_objects::RequestBodyFile;
    use glib::{subclass::InitializingObject, BindingGroup, Properties};
    use gtk::CompositeTemplate;

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::File)]
    #[template(resource = "/es/danirod/Cartero/file_body_pane.ui")]
    pub struct File {
        #[property(get, set)]
        file: RefCell<RequestBodyFile>,
        #[property(get, set)]
        saved: RefCell<bool>,

        #[template_child]
        path: TemplateChild<adw::EntryRow>,
        #[template_child]
        content_type: TemplateChild<adw::EntryRow>,

        binding_group: OnceCell<BindingGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for File {
        const NAME: &'static str = "CarteroFileBodyPane";
        type Type = super::File;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for File {
        fn constructed(&self) {
            self.parent_constructed();

            self.get_binding_group()
                .set_source(Some(&self.obj().file()));
            self.obj().connect_file_notify(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |pane| {
                    imp.get_binding_group().set_source(Some(&pane.file()));
                }
            ));
        }
    }

    impl WidgetImpl for File {}

    impl BoxImpl for File {}

    #[gtk::template_callbacks]
    impl File {
        fn get_binding_group(&self) -> BindingGroup {
            self.binding_group
                .get_or_init(|| {
                    let binding_group = BindingGroup::new();
                    binding_group
                        .bind("path", &*self.path, "text")
                        .bidirectional()
                        .sync_create()
                        .build();
                    binding_group
                        .bind("content_type", &*self.content_type, "text")
                        .bidirectional()
                        .sync_create()
                        .build();
                    binding_group
                })
                .clone()
        }

        fn grab_current_file(&self) -> Option<gtk::gio::File> {
            self.obj()
                .ancestor(EndpointPane::static_type())
                .and_downcast::<EndpointPane>()
                .and_then(|pane| pane.file())
        }

        async fn on_select_file_callback(&self) {
            let Some(prefix_dir) = self.grab_current_file().and_then(|file| file.parent()) else {
                gtk::glib::g_info!(
                    "Cartero",
                    "on_select_file_callback() called for an anonymous file. Aborting..."
                );
                return;
            };
            let root = self.obj().root();
            let window = root.and_downcast::<CarteroWindow>().unwrap();
            let file = pick_file(&window, Some(&prefix_dir)).await;

            match file {
                Ok(Some(file)) => {
                    let relative_path = prefix_dir
                        .relative_path(&file)
                        .and_then(|path| path.to_str().map(|s| s.to_string()))
                        .expect("relative_path mismatch");
                    self.obj().file().set_path(relative_path.as_ref());
                    let file_contents = file
                        .load_bytes_future()
                        .await
                        .map(|(bytes, _)| bytes.to_vec())
                        .unwrap_or_default();
                    let (mime, _) = gtk::gio::content_type_guess(Some(file.uri()), &file_contents);
                    self.obj().file().set_content_type(Some(mime));
                }
                Ok(None) => {
                    gtk::glib::g_info!(
                        "Cartero",
                        "on_select_file_callback() did not get any file. Aborting..."
                    );
                }
                Err(e) => {
                    glib_file_dialog_error(&window, &e).await;
                }
            }
        }

        #[template_callback]
        fn on_select_file(&self) {
            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                async move {
                    pane.on_select_file_callback().await;
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct File(ObjectSubclass<imp::File>)
        @extends gtk::Widget, gtk::Box;
}
