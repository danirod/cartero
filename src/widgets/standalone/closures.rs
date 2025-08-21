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

use gettextrs::gettext;
use gtk::gio;
use gtk::prelude::*;
use gtk::ClosureExpression;

use crate::widgets::shell::BasePane;

pub fn install_base_pane_page_closures(page: &adw::TabPage, child: &BasePane) {
    // Define the bindings that act on the tab indicator.
    let file = child.property_expression("file");
    let dirty = child.property_expression("dirty");

    // Bind title
    ClosureExpression::with_callback([&file, &dirty], |args| {
        let file = args[1].get::<Option<gio::File>>().unwrap();
        let dirty = args[2].get::<bool>().unwrap();
        let path = file
            .and_then(|f| f.basename())
            .map(|bn| bn.file_stem().unwrap().to_str().unwrap().to_string())
            .unwrap_or(gettext("(untitled)"));
        if dirty {
            format!("• {}", &path)
        } else {
            path
        }
    })
    .bind(page, "title", Some(child));

    // Bind subtitle
    ClosureExpression::with_callback([&file], |args| {
        let file = args[1].get::<Option<gio::File>>().unwrap();
        file.and_then(|f| f.path())
            .map(|bn| bn.display().to_string())
            .unwrap_or(gettext("Draft"))
    })
    .bind(page, "tooltip", Some(child));
}
