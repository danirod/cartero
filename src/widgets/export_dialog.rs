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

    use crate::widgets::dialogs::glib_file_dialog_error;

    use super::*;
    use formatx::formatx;
    use gettextrs::gettext;
    use glib::{subclass::InitializingObject, Properties};
    use gtk::{
        gdk::{ContentProvider, Display},
        CompositeTemplate,
    };

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::ExportDialog)]
    #[template(resource = "/es/danirod/Cartero/export_dialog.ui")]
    pub struct ExportDialog {
        #[template_child]
        buffer: TemplateChild<sourceview5::Buffer>,

        #[template_child]
        toaster: TemplateChild<adw::ToastOverlay>,

        #[property(get, set)]
        blob: RefCell<Option<glib::Bytes>>,

        #[property(get, set)]
        format: RefCell<Option<sourceview5::Language>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExportDialog {
        const NAME: &'static str = "CarteroExportDialog";
        type Type = super::ExportDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ExportDialog {
        fn constructed(&self) {
            self.parent_constructed();

            self.preview_blob();
            let obj = self.obj();
            obj.connect_blob_notify(glib::clone!(
                #[weak(rename_to = dialog)]
                self,
                move |_| {
                    dialog.preview_blob();
                }
            ));
        }
    }

    impl WidgetImpl for ExportDialog {}

    impl AdwDialogImpl for ExportDialog {}

    #[gtk::template_callbacks]
    impl ExportDialog {
        fn preview_blob(&self) {
            let obj = &*self.obj();
            let content = obj
                .blob()
                .clone()
                .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                .unwrap_or_default();
            self.buffer.set_text(&content);
        }

        #[template_callback]
        fn on_copy_button_clicked(&self) {
            let obj = &*self.obj();
            let content = {
                let blob = obj.blob().unwrap_or(glib::Bytes::from(&[]));
                ContentProvider::new_union(&[
                    ContentProvider::for_bytes("text/plain", &blob),
                    ContentProvider::for_bytes("text/plain;charset=utf-8", &blob),
                ])
            };
            if let Some(display) = Display::default() {
                let clipboard = display.clipboard();
                clipboard.set_content(Some(&content)).unwrap();

                let msg = gettext("Content copied to the clipboard");
                let toast = adw::Toast::new(&msg);
                self.toaster.add_toast(toast);
            }
        }

        #[template_callback]
        fn on_save_button_clicked(&self) {
            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = dialog)]
                self,
                async move {
                    let obj = &*dialog.obj();
                    let root = obj.root().and_downcast::<gtk::Window>().unwrap();
                    let file = match crate::widgets::export_file(&root, None).await {
                        Ok(maybe_file) => maybe_file,
                        Err(e) => {
                            glib_file_dialog_error(&root, &e).await;
                            None
                        }
                    };
                    if let Some(file) = file {
                        let blob = obj.blob().unwrap_or(glib::Bytes::from(&[]));
                        let result = file
                            .replace_contents_future(
                                blob,
                                None,
                                false,
                                gtk::gio::FileCreateFlags::NONE,
                            )
                            .await;
                        match result {
                            Ok(_) => {
                                let basename = file
                                    .basename()
                                    .map(|bn| bn.to_str().unwrap().to_string())
                                    .unwrap();
                                let msg = formatx!(gettext("Content saved as file {}"), basename)
                                    .unwrap();
                                let toast = adw::Toast::new(&msg);
                                dialog.toaster.add_toast(toast);
                            }
                            Err((_, e)) => {
                                glib_file_dialog_error(&root, &e).await;
                            }
                        }
                    };
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct ExportDialog(ObjectSubclass<imp::ExportDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
