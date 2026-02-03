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

use adw::{
    AlertDialog,
    prelude::{AlertDialogExt, AlertDialogExtManual},
};
use formatx::formatx;
use gettextrs::gettext;
use glib::object::IsA;

pub enum AppUpdateDialogResponse {
    OpenWebsite,
    Skip,
}

pub async fn notify_check_update_error(root: &impl IsA<gtk::Widget>) {
    let dialog = AlertDialog::builder()
        .heading(&gettext("Error checking updates"))
        .body(&gettext(
            "Couldn't receive a valid response from the server. Please, check later.",
        ))
        .build();
    dialog.add_responses(&[("accept", &gettext("Accept"))]);
    dialog.set_response_appearance("open", adw::ResponseAppearance::Suggested);
    dialog.choose_future(root).await;
}

pub async fn notify_latest_version(root: &impl IsA<gtk::Widget>) {
    let dialog = AlertDialog::builder()
        .heading(&gettext("No updates at the moment"))
        .body(&gettext("You are running the latest version of Cartero."))
        .build();
    dialog.add_responses(&[("accept", &gettext("Accept"))]);
    dialog.set_response_appearance("open", adw::ResponseAppearance::Suggested);
    dialog.choose_future(root).await;
}

pub async fn notify_app_available(
    root: &impl IsA<gtk::Widget>,
    version: &str,
) -> AppUpdateDialogResponse {
    let body = formatx!(gettext("Version {} has been released. Do you want to visit the website to download the update now?"), &version).unwrap();
    let question = AlertDialog::builder()
        .heading(&gettext("There is a new version of Cartero!"))
        .body(&body)
        .build();
    question.add_responses(&[
        ("open", &gettext("_Open website")),
        ("skip", &gettext("_Not now")),
    ]);
    question.set_response_appearance("open", adw::ResponseAppearance::Suggested);
    let response = question.choose_future(root).await;
    match response.as_str() {
        "open" => AppUpdateDialogResponse::OpenWebsite,
        _ => AppUpdateDialogResponse::Skip,
    }
}
