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

use crate::{
    error::{FileLoadError, FileSaveError},
    file::FileWarningTag,
};

use adw::{
    prelude::{AlertDialogExt, AlertDialogExtManual},
    AlertDialog,
};
use formatx::formatx;
use gettextrs::gettext;
use gio::prelude::FileExt;
use glib::object::IsA;
use gtk::gio;

fn get_file_display_name(file: &gio::File) -> Option<String> {
    file.basename()
        .map(|bn| bn.file_stem().unwrap().to_str().unwrap().to_string())
}

/// Renders an error message that shows the reason on why the given file could
/// not be opened. The path to the file to load should be given as an argument
/// so that it can be presented in the title.
pub async fn file_load_error_dialog(
    root: &impl IsA<gtk::Widget>,
    file: Option<&gio::File>,
    error: &FileLoadError,
) {
    let file_name = file.and_then(get_file_display_name);
    let error_title = match file_name {
        Some(name) => formatx!(gettext("Cannot open '{}'"), name).unwrap(),
        None => gettext("Cannot open the requested file"),
    };
    let error_msg = error.to_string();

    let alert = AlertDialog::builder()
        .heading(&error_title)
        .body(&error_msg)
        .default_response("close")
        .build();
    alert.add_response("close", &gettext("Close"));
    alert.choose_future(root).await;
}

/// Renders an error message that shows the warnings found while loading a file.
/// The path to the file to load should be given as an argument so that it can
/// be presented in the title.
pub async fn file_load_warning_dialog(
    root: &impl IsA<gtk::Widget>,
    file: Option<&gio::File>,
    failures: &[FileWarningTag],
) {
    let file_name = file.and_then(get_file_display_name);
    let error_title = match file_name {
        Some(name) => formatx!(gettext("The file '{}' was loaded but had issues"), name).unwrap(),
        None => gettext("The requested file was loaded but had issues"),
    };
    let error_msg = if failures.len() > 1 {
        failures
            .iter()
            .map(|w| format!("• {w}"))
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        failures[0].to_string()
    };
    let alert = AlertDialog::builder()
        .heading(&error_title)
        .body(&error_msg)
        .default_response("close")
        .build();
    alert.add_response("close", &gettext("Close"));
    alert.choose_future(root).await;
}

/// Renders an error message that shows the error that prevents the file from
/// being saved. The path to the file to save should be given as an argument so
/// that it can be presented in the title.
pub async fn file_save_error(
    root: &impl IsA<gtk::Widget>,
    file: Option<&gio::File>,
    error: FileSaveError,
) {
    let file_name = file.and_then(get_file_display_name);
    let error_title = match file_name {
        Some(name) => formatx!(gettext("Cannot save '{}'"), name).unwrap(),
        None => gettext("Cannot save the requested file"),
    };
    let error_msg = format!("{}\n\n{}", gettext("There was an error during the saving process. Assume that your changes are still not saved."), error);
    let alert = AlertDialog::builder()
        .heading(&error_title)
        .body(&error_msg)
        .default_response("close")
        .build();
    alert.add_response("close", &gettext("Close"));
    alert.choose_future(root).await;
}

/// Renders an error message associated with a FileDialog.
pub async fn glib_file_dialog_error(root: &impl IsA<gtk::Widget>, error: &glib::Error) {
    let alert = AlertDialog::builder()
        .heading(&gettext(
            "Could not select a valid file from the file chooser",
        ))
        .body(&error.to_string())
        .default_response("close")
        .build();
    alert.add_response("close", &gettext("Close"));
    alert.choose_future(root).await;
}

// Asks the user whether to actually close the window when there is unsaved
// data. Returns true if the user chose to close the window and false if
// chose to keep the window for now.
pub async fn confirm_close_window(root: &impl IsA<gtk::Widget>) -> bool {
    let question = AlertDialog::builder()
        .heading(&gettext("There are unsaved changes"))
        .body(&gettext(
            "Closing this window will lose all unsaved data. Do you really want to proceed?",
        ))
        .build();
    question.add_responses(&[
        ("cancel", &gettext("Cancel")),
        ("continue", &gettext("Close anyway")),
    ]);
    question.set_response_appearance("continue", adw::ResponseAppearance::Destructive);
    question.set_default_response(Some("cancel"));
    let response = question.choose_future(root).await;
    response == "continue"
}

pub enum SaveAlertDialogResponse {
    Cancel,
    Discard,
    Save,
}

/// Asks the user whether to save data. This is used when the user tries to close
/// a pane with modifications. It returns an enum value that indicates the answer
/// to the question.
pub async fn confirm_save(
    root: &impl IsA<gtk::Widget>,
    file: Option<&gio::File>,
) -> SaveAlertDialogResponse {
    let file_name = file.and_then(get_file_display_name);
    let question_title = match file_name {
        Some(name) => formatx!(gettext("Save changes in '{}'?"), name).unwrap(),
        None => gettext("Save changes?"),
    };
    let question = AlertDialog::builder()
        .heading(&question_title)
        .body(&gettext(
            "There are changes that have not been saved yet. What do you want to do?",
        ))
        .build();
    question.add_responses(&[
        ("cancel", &gettext("_Cancel")),
        ("discard", &gettext("_Discard")),
        ("save", &gettext("_Save")),
    ]);
    question.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    question.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    let response = question.choose_future(root).await;
    match response.as_str() {
        "discard" => SaveAlertDialogResponse::Discard,
        "save" => SaveAlertDialogResponse::Save,
        _ => SaveAlertDialogResponse::Cancel,
    }
}
