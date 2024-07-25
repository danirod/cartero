use gettextrs::gettext;
use glib::types::StaticType;
use gtk::{gio::ListStore, prelude::FileExt, DialogError, FileDialog, FileFilter};
use std::path::PathBuf;

use crate::{error::CarteroError, win::CarteroWindow};

fn get_cartero_file_filter() -> FileFilter {
    let filter = FileFilter::new();
    if cfg!(target_os = "windows") {
        filter.add_pattern("*.cartero");
    } else {
        filter.add_mime_type("application/cartero");
    }
    if cfg!(target_os = "macos") {
        filter.add_suffix(".cartero");
    }
    filter.set_name(Some(&gettext("Request (.cartero)")));
    filter
}

pub async fn open_file(win: &CarteroWindow) -> Result<PathBuf, CarteroError> {
    let filters = ListStore::with_type(FileFilter::static_type());
    let cartero = get_cartero_file_filter();
    filters.append(&cartero);

    let dialog = FileDialog::builder()
        .accept_label(&gettext("Open"))
        .title(&gettext("Open request"))
        .filters(&filters)
        .default_filter(&cartero)
        .modal(true)
        .build();
    let file = dialog.open_future(Some(win)).await.map_err(|e| {
        if let Some(file_error) = e.kind::<DialogError>() {
            match file_error {
                DialogError::Dismissed => CarteroError::NoFilePicked,
                _ => CarteroError::FileDialogError,
            }
        } else {
            CarteroError::FileDialogError
        }
    })?;
    let path = file.path().ok_or(CarteroError::FileDialogError)?;
    Ok(path)
}

pub async fn save_file(win: &CarteroWindow) -> Result<PathBuf, CarteroError> {
    let filters = ListStore::with_type(FileFilter::static_type());
    let cartero = get_cartero_file_filter();
    filters.append(&cartero);

    let dialog = FileDialog::builder()
        .accept_label(&gettext("Save"))
        .title(&gettext("Save request"))
        .modal(true)
        .filters(&filters)
        .default_filter(&cartero)
        .build();
    let file = dialog.save_future(Some(win)).await.map_err(|e| {
        if let Some(file_error) = e.kind::<DialogError>() {
            match file_error {
                DialogError::Dismissed => CarteroError::NoFilePicked,
                _ => CarteroError::FileDialogError,
            }
        } else {
            CarteroError::FileDialogError
        }
    })?;
    let path = file.path().ok_or(CarteroError::FileDialogError)?;
    Ok(path)
}
