use gettextrs::gettext;
use gtk::{prelude::FileExt, DialogError, FileDialog};
use std::path::PathBuf;

use crate::{error::CarteroError, win::CarteroWindow};

pub async fn open_file(win: &CarteroWindow) -> Result<PathBuf, CarteroError> {
    let dialog = FileDialog::builder()
        .accept_label(&gettext("Open"))
        .title(&gettext("Open request"))
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
    let dialog = FileDialog::builder()
        .accept_label(&gettext("Save"))
        .title(&gettext("Save request"))
        .modal(true)
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
