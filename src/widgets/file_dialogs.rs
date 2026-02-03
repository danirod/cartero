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

use formatx::formatx;
use gettextrs::gettext;
use glib::{object::IsA, prelude::Cast, types::StaticType};
use gtk::{
    DialogError, FileDialog, FileFilter,
    gio::{self, ListStore},
    prelude::{FileExt, ListModelExtManual, SettingsExtManual},
};
use std::path::PathBuf;

use crate::{settings::Settings, win::CarteroWindow};

use super::dialogs::file_pick_out_of_prefix_error;

/// Creates a brand new file dialog with the filters already set. Note that
/// there is still no title or button text. Make sure to set the values before
/// presenting the dialog.
fn new_file_dialog() -> FileDialog {
    let filters = ListStore::with_type(FileFilter::static_type());
    let filter = {
        let filter = FileFilter::new();
        filter.add_pattern("*.cartero");

        // TRANSLATORS: presented in file dialogs as the file type for a request
        let file_format = gettext("Request");
        if cfg!(not(target_os = "windows")) {
            filter.add_mime_type("application/cartero");
            filter.add_suffix("cartero");

            let file_format = formatx!("{} (.cartero)", file_format).unwrap();
            filter.set_name(Some(&file_format));
        } else {
            filter.set_name(Some(&file_format));
        }
        filter
    };
    filters.append(&filter);

    FileDialog::builder()
        .filters(&filters)
        .default_filter(&filter)
        .modal(true)
        .build()
}

/// The settings key to use where the directory of the last opened file is.
/// The next time the open file dialog is called, this is where we start.
const LAST_OPEN_DIR: &'static str = "last-open-dir";

/// The settings key to use where the directory of the last saved file is.
/// The next time the save file dialog is called, this is where we start.
const LAST_SAVE_DIR: &'static str = "last-save-dir";

/// Returns the gio::File for the path stored in the settings under the key.
fn get_file_setting(key: &str) -> Option<gio::File> {
    let settings = Settings::default();
    settings.get::<Option<String>>(key).map(|path| {
        let path = PathBuf::from(&path);
        gio::File::for_path(path)
    })
}

/// Saves the path to the given file in the settings under the assigned key.
/// It the given file is None, then the assigned key will be reset to the
/// default value.
fn set_file_setting(key: &str, value: Option<&gio::File>) {
    let settings = Settings::default();
    let path = value
        .and_then(|file| file.path())
        .and_then(|path| path.to_str().map(String::from));
    if let Err(e) = settings.set(key, path) {
        glib::g_warning!(
            "Cartero",
            "Couldn't persist a new value for key {}: {}",
            key,
            e
        );
    }
}

/// Opens a GTK open file dialog that can be used to open endpoint files. The
/// list of endpoint files opened will be returned. If the user cancels the
/// operation, an empty list will be returned. This function returns a Result,
/// but the conditions on which the file dialog will actually fail are obscure.
pub async fn open_files(win: &CarteroWindow) -> Result<Vec<gio::File>, glib::Error> {
    /* Create the open dialog. */
    let dialog = new_file_dialog();
    dialog.set_accept_label(Some(&gettext("Open")));
    dialog.set_title(&gettext("Open request"));
    dialog.set_initial_folder(get_file_setting(LAST_OPEN_DIR).as_ref());

    let files = match dialog.open_multiple_future(Some(win)).await {
        Ok(result) => Ok(result),
        Err(e) => match e.kind::<DialogError>() {
            /* The dialog treats cancellation or dismission as an error. Swallow the error in that case. */
            Some(DialogError::Cancelled | DialogError::Dismissed) => {
                glib::g_info!("Cartero", "File open dialog cancelled by user");
                return Ok(Vec::new());
            }
            _ => Err(e),
        },
    }?;

    let files: Vec<gio::File> = files
        .snapshot()
        .into_iter()
        .map(|obj| obj.downcast::<gio::File>().unwrap())
        .collect::<Vec<gio::File>>();

    /* Update the last open dir, but only if files were actually picked. */
    if let Some(first_file) = files.first() {
        set_file_setting(LAST_OPEN_DIR, first_file.parent().as_ref());
    }
    Ok(files)
}

/// Opens a GTK save file dialog that can be used to pick a save location.
/// The file that the user has requested to use will be returned. If the user
/// cancels the operation, the returned result will be empty. The conditions
/// for a glib::Error are obscure.
pub async fn save_file(win: &CarteroWindow) -> Result<Option<gio::File>, glib::Error> {
    /* Create the save dialog. */
    let dialog = new_file_dialog();
    dialog.set_accept_label(Some(&gettext("Save")));
    dialog.set_title(&gettext("Save request"));
    dialog.set_initial_folder(get_file_setting(LAST_SAVE_DIR).as_ref());
    let initial_file_name = if cfg!(target_os = "macos") {
        // The macOS native file dialog will already add the extension for us.
        gettext("Request")
    } else {
        formatx!("{}.cartero", gettext("Request")).unwrap()
    };
    dialog.set_initial_name(Some(&initial_file_name));

    let file = match dialog.save_future(Some(win)).await {
        Ok(result) => Ok(Some(result)),
        Err(e) => match e.kind::<DialogError>() {
            /* The dialog treats cancellation or dismission as an error. Swallow the error in that case. */
            Some(DialogError::Cancelled | DialogError::Dismissed) => {
                glib::g_info!("Cartero", "File save dialog cancelled by user");
                return Ok(None);
            }
            _ => Err(e),
        },
    }?;

    /* Update the last save dir, but only if a file was actually picked. */
    if let Some(file) = &file {
        set_file_setting(LAST_SAVE_DIR, file.parent().as_ref());
    }
    Ok(file)
}

/// Displays a file picker that lets the user pick a file for opening purposes. A prefix may be
/// given as an extra path. If set, the picked file must have the same prefix; in other words, it
/// has to be part of the directory prefix_path, or it has to be in a subdirectory of prefix_path.
pub async fn pick_file<T>(
    parent: &T,
    prefix_path: Option<&gio::File>,
) -> Result<Option<gio::File>, glib::Error>
where
    T: IsA<gtk::Window> + Clone + 'static,
{
    let dialog = FileDialog::builder().modal(true).build();
    dialog.set_accept_label(Some(&gettext("Open")));
    dialog.set_title(&gettext("Select file"));
    dialog.set_initial_folder(prefix_path);

    let file = match dialog.open_future(Some(parent)).await {
        Ok(result) => {
            let in_prefix = match prefix_path {
                None => true,
                Some(prefix) => result.has_prefix(prefix),
            };
            if in_prefix {
                Ok(Some(result))
            } else {
                file_pick_out_of_prefix_error(parent.upcast_ref(), &result, &prefix_path.unwrap())
                    .await;
                Ok(None)
            }
        }
        Err(e) => match e.kind::<DialogError>() {
            /* The dialog treats cancellation or dismission as an error. Swallow the error in that case. */
            Some(DialogError::Cancelled | DialogError::Dismissed) => {
                glib::g_info!("Cartero", "File save dialog cancelled by user");
                return Ok(None);
            }
            _ => Err(e),
        },
    }?;

    Ok(file)
}

/// Opens a generic export dialog that can be used to pick a save location.
/// This dialog will accept other file types that are not the application
/// type.
pub async fn export_file<T>(
    parent: &T,
    initial_name: Option<&str>,
) -> Result<Option<gio::File>, glib::Error>
where
    T: IsA<gtk::Window> + Clone + 'static,
{
    let dialog = FileDialog::builder().modal(true).build();
    dialog.set_accept_label(Some(&gettext("Save")));
    dialog.set_initial_name(initial_name);
    dialog.set_title(&gettext("Export to file"));

    let file = match dialog.save_future(Some(parent)).await {
        Ok(result) => Ok(Some(result)),
        Err(e) => match e.kind::<DialogError>() {
            /* The dialog treats cancellation or dismission as an error. Swallow the error in that case. */
            Some(DialogError::Cancelled | DialogError::Dismissed) => {
                glib::g_info!("Cartero", "File save dialog cancelled by user");
                return Ok(None);
            }
            _ => Err(e),
        },
    }?;

    // TODO: Save the last export directory
    Ok(file)
}
