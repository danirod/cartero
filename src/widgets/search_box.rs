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

use glib::subclass::types::ObjectSubclassIsExt;
use gtk::{
    glib,
    prelude::{EditableExt, WidgetExt},
};
use sourceview5::prelude::SearchSettingsExt;

mod imp {
    use std::cell::{OnceCell, RefCell};
    use std::sync::OnceLock;

    use glib::object::{Cast, ObjectExt};
    use glib::subclass::{InitializingObject, Signal};
    use glib::Properties;
    use gtk::gio::{ActionEntry, SimpleActionGroup};
    use gtk::prelude::{ActionMapExtManual, EditableExt, TextBufferExt, TextViewExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtk::{gdk, glib, TextIter};
    use gtk::{CompositeTemplate, TemplateChild};
    use sourceview5::prelude::SearchSettingsExt;
    use sourceview5::{Buffer, SearchContext, SearchSettings};

    use crate::i18n::{i18n_f, ni18n_f};
    use crate::widgets::CodeView;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/es/danirod/Cartero/search_box.ui")]
    #[properties(wrapper_type = super::SearchBox)]
    pub struct SearchBox {
        #[property(get, construct_only)]
        editable: RefCell<CodeView>,

        #[template_child]
        pub(super) search_content: TemplateChild<gtk::Text>,

        #[template_child]
        search_previous: TemplateChild<gtk::Button>,

        #[template_child]
        search_next: TemplateChild<gtk::Button>,

        #[template_child]
        search_close: TemplateChild<gtk::Button>,

        #[template_child]
        search_results: TemplateChild<gtk::Label>,

        search_context: OnceCell<SearchContext>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchBox {
        const NAME: &'static str = "CarteroSearchBox";
        type Type = super::SearchBox;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.add_binding_action(gdk::Key::F, gdk::ModifierType::CONTROL_MASK, "search.focus");
            klass.add_binding_action(gdk::Key::Escape, gdk::ModifierType::empty(), "search.close");
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SearchBox {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("close").build()])
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.init_actions();
        }
    }

    impl WidgetImpl for SearchBox {}

    impl BoxImpl for SearchBox {}

    #[gtk::template_callbacks]
    impl SearchBox {
        pub fn get_search_context(&self) -> &SearchContext {
            self.search_context.get_or_init(|| {
                let editable = self.editable.borrow();
                let buffer = editable.buffer().downcast::<Buffer>().unwrap();
                let settings = SearchSettings::builder().wrap_around(true).build();
                let context = SearchContext::builder()
                    .buffer(&buffer)
                    .highlight(true)
                    .settings(&settings)
                    .build();
                context.connect_occurrences_count_notify(
                    glib::clone!(@weak self as imp => move |_| {
                        imp.update_search_ocurrences();
                    }),
                );
                context
            })
        }

        fn init_actions(&self) {
            let obj = self.obj();
            let action_previous = ActionEntry::builder("previous")
                .activate(glib::clone!(@weak self as imp => move |_, _, _| {
                    imp.search_backward();
                }))
                .build();
            let action_next = ActionEntry::builder("next")
                .activate(glib::clone!(@weak self as imp => move |_, _, _| {
                    imp.search_forward();
                }))
                .build();
            let action_close = ActionEntry::builder("close")
                .activate(glib::clone!(@weak self as imp => move |_, _, _| {
                    imp.close_search();
                }))
                .build();
            let action_focus = ActionEntry::builder("focus")
                .activate(glib::clone!(@weak obj => move |_, _, _| {
                    obj.focus();
                }))
                .build();
            let group = SimpleActionGroup::new();
            group.add_action_entries([action_previous, action_next, action_close, action_focus]);
            obj.insert_action_group("search", Some(&group));
        }

        fn editable_iter(&self, forward: bool) -> TextIter {
            let editable = self.editable.borrow();
            let buffer = editable.buffer();
            match buffer.selection_bounds() {
                Some((start, end)) => {
                    if forward {
                        end
                    } else {
                        start
                    }
                }
                None => {
                    let offset = buffer.cursor_position();
                    buffer.iter_at_offset(offset)
                }
            }
        }

        fn search_forward(&self) {
            let iter = self.editable_iter(true);
            let editable = self.editable.borrow();
            let buffer = editable.buffer();
            let search_context = self.get_search_context();

            if let Some((start, end, _wrapped)) = search_context.forward(&iter) {
                buffer.select_range(&start, &end);
                editable.scroll_mark_onscreen(&buffer.get_insert());
            }

            self.update_search_ocurrences();
        }

        fn search_backward(&self) {
            let iter = self.editable_iter(false);
            let editable = self.editable.borrow();
            let search_context = self.get_search_context();
            let buffer = editable.buffer();

            if let Some((start, end, _wrapped)) = search_context.backward(&iter) {
                buffer.select_range(&start, &end);
                editable.scroll_mark_onscreen(&buffer.get_insert());
            }

            self.update_search_ocurrences();
        }

        #[template_callback]
        fn on_text_changed(&self, entry: &gtk::Text) {
            let text = entry.text();
            let search_context = self.get_search_context();
            search_context.settings().set_search_text(Some(&text));

            // Make an initial search to see if there are results.
            let iter = self.editable_iter(true);
            let found = search_context.forward(&iter).is_some();
            self.search_next.set_sensitive(found);
            self.search_previous.set_sensitive(found);

            self.update_search_ocurrences();
        }

        #[template_callback]
        fn on_text_activate(&self) {
            self.search_forward();
        }

        fn update_search_ocurrences(&self) {
            let editable = self.editable.borrow();
            let buffer = editable.buffer();
            let search_context = self.get_search_context();

            let total = search_context.occurrences_count();
            let current = match buffer.selection_bounds() {
                Some((start, end)) => search_context.occurrence_position(&start, &end),
                None => -1,
            };

            if total >= 1 && current >= 1 {
                let total = format!("{total}");
                let current = format!("{current}");

                // TRANSLATORS: this string is used to build the search box ocurrences count; the first
                // placeholder is the current ocurrence index, the second one is the total.
                let label = i18n_f("{} of {}", &[&current, &total]);
                self.search_results.set_label(&label);
            } else if total >= 1 {
                let utotal = total as u32;
                let total = format!("{total}");

                // TRANSLATORS: this string is used to build the search box ocurrences count when only
                // the number of total ocurrences is known.
                let label = ni18n_f("{} result", "{} results", utotal, &[&total]);
                self.search_results.set_label(&label);
            } else {
                self.search_results.set_label("");
            }
        }

        fn close_search(&self) {
            let search_context = self.get_search_context();
            search_context.settings().set_search_text(None);
            let obj = self.obj();
            obj.emit_by_name::<()>("close", &[]);
        }
    }
}

glib::wrapper! {
    pub struct SearchBox(ObjectSubclass<imp::SearchBox>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable;
}

impl SearchBox {
    pub fn focus(&self) {
        let imp = self.imp();
        imp.search_content.grab_focus();
    }

    pub fn init_search(&self, text: Option<&str>) {
        let imp = self.imp();
        let content = &*imp.search_content;
        let search_context = imp.get_search_context();

        if let Some(text) = text {
            content.set_text(text);
        }

        let search_text: String = content.text().into();
        search_context
            .settings()
            .set_search_text(Some(&search_text));
    }
}
