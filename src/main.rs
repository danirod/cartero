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

#![windows_subsystem = "windows"]

mod app;
mod client;
mod error;
mod file;
mod widgets;
#[rustfmt::skip]
mod config;
mod entities;
mod objects;
mod utils;
mod win;

use std::path::PathBuf;

use gettextrs::LocaleCategory;
use gtk::gio;
use gtk::prelude::*;

use self::app::CarteroApplication;
use self::config::{APP_ID, GETTEXT_PACKAGE};

fn app_rel_path(dir: &str) -> PathBuf {
    let root_dir = std::env::current_exe()
        .map(|p| p.parent().unwrap().parent().unwrap().to_path_buf())
        .unwrap();
    #[cfg(target_os = "macos")]
    let root_dir = {
        // Still don't hardcode the Resources directory so that build-aux/cargo-build.sh still works on macOS.
        let resources_dir = root_dir.join("Resources");
        if resources_dir.exists() && resources_dir.is_dir() {
            resources_dir
        } else {
            root_dir
        }
    };
    root_dir.join(dir)
}

fn init_data_dir() {
    let datadir = app_rel_path("share");
    let xdg_data_dirs: Vec<PathBuf> = match std::env::var("XDG_DATA_DIRS") {
        Ok(dirs) => std::env::split_paths(&dirs).collect(),
        Err(_) => vec![],
    };
    if !xdg_data_dirs.iter().any(|d| d == &datadir) {
        let mut xdg_final_dirs = vec![datadir];
        xdg_final_dirs.extend(xdg_data_dirs);
        let xdg_data_dir = std::env::join_paths(&xdg_final_dirs).unwrap();
        std::env::set_var("XDG_DATA_DIRS", xdg_data_dir);
    }
}

fn init_locale() {
    let localedir = app_rel_path("share/locale");
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, localedir).expect("Unable to bind the text domain");
    gettextrs::bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8").unwrap();
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");
}

fn init_glib() {
    glib::set_application_name(APP_ID);
}

fn init_gio_resources() {
    let resource_file = app_rel_path("share/cartero/cartero.gresource");
    let res = gio::Resource::load(resource_file).expect("Could not load gresource file");
    gio::resources_register(&res);
}

fn main() -> glib::ExitCode {
    #[cfg(target_os = "windows")]
    {
        if let Err(_) = std::env::var("GSK_RENDERER") {
            std::env::set_var("GSK_RENDERER", "cairo");
        }
        std::env::set_var("GTK_CSD", "0");
    }

    #[cfg(target_os = "macos")]
    {
        let gdk_pixbuf = app_rel_path("lib/gdk-pixbuf-2.0/2.10.0/loaders.cache");
        if let Ok(true) = gdk_pixbuf.try_exists() {
            std::env::set_var("GDK_PIXBUF_MODULE_FILE", gdk_pixbuf);
        }
    }

    init_data_dir();
    init_locale();
    init_glib();
    init_gio_resources();

    // This is dirty, but because adw_init() calls bindtextdomain() and uses a hardcoded static
    // path, I need to actually re-bind libadwaita against my own localedir on platforms where the
    // datadir is not fixed, so that it can use a path relative to the application executable
    // again.
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        adw::init().expect("Failed to initialize system runtimes");
        let localedir = app_rel_path("share/locale");
        gettextrs::bindtextdomain("libadwaita", localedir).expect("Unable to bind the text domain");
    }

    let app = CarteroApplication::new();
    app.run()
}

/// This function does some nasty things in order to get the Gio resources file loaded during tests.
/// It is closely based on what gio::resources_register_include!() macro does, but without requiring
/// to set the OUT_DIR variable. However, make sure to build the gresource file first into build.
#[cfg(test)]
pub fn init_test_resources() {
    let bytes = glib::Bytes::from_static(include_bytes!("../build/data/cartero.gresource"));
    let resource = gio::Resource::from_data(&bytes).unwrap();
    gio::resources_register(&resource);
}
