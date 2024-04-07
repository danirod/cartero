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

mod app;
mod client;
mod widgets;
#[rustfmt::skip]
mod config;
mod objects;
mod win;

use gettextrs::LocaleCategory;
use gtk4::gio;
use gtk4::prelude::*;

use self::app::CarteroApplication;
use self::config::GETTEXT_PACKAGE;

fn main() -> glib::ExitCode {
    // Infer the location of DATADIR and PKGDATADIR from the executable location
    let exe = std::env::current_exe().expect("Cannot get current_exe() for app");
    let path = exe
        .parent()
        .and_then(|p| p.to_str())
        .expect("Cannot get current_exe() location");
    let locale_dir = format!("{}/../share/locale", path);
    let resource_file = format!("{}/../share/cartero/cartero.gresource", path);

    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, locale_dir).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let res = gio::Resource::load(resource_file).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = CarteroApplication::new();
    app.run()
}
