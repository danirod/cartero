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

use std::sync::OnceLock;

use glib::object::{CastNone, ObjectExt};
use gtk::subclass::prelude::*;

use crate::entities::RequestExportType;

use super::{BaseExportPaneExt, CodeExportPane};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "CarteroExportType")]
pub enum ExportType {
    #[default]
    None,
    Curl,
}

impl ExportType {
    pub fn types() -> &'static [ExportType] {
        static TYPES: OnceLock<Vec<ExportType>> = OnceLock::new();
        TYPES.get_or_init(|| vec![ExportType::None, ExportType::Curl])
    }
}

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use adw::ComboRow;
    use glib::subclass::Signal;
    use glib::{subclass::InitializingObject, Properties};
    use gtk::template_callbacks;
    use gtk::Separator;
    use gtk::{CompositeTemplate, Stack};

    use crate::widgets::BaseExportPane;
    use crate::widgets::CodeExportPane;

    use super::ExportType;

    #[derive(Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::ExportTab)]
    #[template(resource = "/es/danirod/Cartero/export_tab.ui")]
    pub struct ExportTab {
        #[template_child]
        stack: TemplateChild<Stack>,

        #[template_child]
        combo: TemplateChild<ComboRow>,

        #[template_child]
        sep: TemplateChild<Separator>,

        #[template_child]
        code: TemplateChild<CodeExportPane>,

        #[property(get = Self::export_type, set = Self::set_export_type, builder(ExportType::default()))]
        _payload_type: RefCell<ExportType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExportTab {
        const NAME: &'static str = "CarteroExportTab";
        type Type = super::ExportTab;
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
    impl ObjectImpl for ExportTab {
        fn constructed(&self) {
            self.parent_constructed();
            self.on_selection_changed();

            self.combo.connect_selected_notify(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }
            ));

            self.code.connect_changed(glib::clone!(
                #[weak(rename_to = pane)]
                self,
                move |_| {
                    pane.obj().emit_by_name::<()>("changed", &[]);
                }
            ));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("changed").build()])
        }
    }

    impl WidgetImpl for ExportTab {}

    impl BoxImpl for ExportTab {}

    #[template_callbacks]
    impl ExportTab {
        #[template_callback]
        fn on_selection_changed(&self) {
            let export_type = self.export_type();
            let tab = match export_type {
                ExportType::None => "none",
                ExportType::Curl => "code",
            };

            self.stack.set_visible_child_name(tab);
            self.sep.set_visible(export_type != ExportType::None);
            self.code.set_format(export_type);
        }

        pub fn export_type(&self) -> ExportType {
            let n_item = self.combo.selected();
            ExportType::types()[n_item as usize]
        }

        pub fn set_export_type(&self, et: ExportType) {
            let pos = ExportType::types().iter().position(|&t| t == et).unwrap();
            self.combo.set_selected(pos as u32);
        }

        pub(super) fn get_active_widget(&self) -> Option<BaseExportPane> {
            match self.export_type() {
                ExportType::None => None,
                ExportType::Curl => Some(self.code.upcast_ref::<BaseExportPane>().clone()),
            }
        }
    }
}

glib::wrapper! {
    pub struct ExportTab(ObjectSubclass<imp::ExportTab>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable;
}

impl ExportTab {
    pub fn connect_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "changed",
            true,
            glib::closure_local!(|ref pane| {
                f(pane);
            }),
        )
    }

    pub fn set_request_export_type(&self, req_export_type: &RequestExportType) {
        let imp = self.imp();

        imp.set_export_type(match req_export_type {
            RequestExportType::None => ExportType::None,
            RequestExportType::Curl(_) => ExportType::Curl,
        });

        let widget = imp.get_active_widget();

        match imp.export_type() {
            ExportType::None => {}
            ExportType::Curl => {
                let widget = widget.and_downcast::<CodeExportPane>().unwrap();
                widget.set_request_export_type(req_export_type);
            }
        }
    }

    pub fn request_export_type(&self) -> RequestExportType {
        let widget = self.imp().get_active_widget();
        let imp = self.imp();

        match imp.export_type() {
            ExportType::None => RequestExportType::None,
            ExportType::Curl => {
                let widget = widget.and_downcast::<CodeExportPane>().unwrap();
                widget.request_export_type()
            }
        }
    }
}
