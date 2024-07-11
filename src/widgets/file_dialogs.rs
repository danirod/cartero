use gettextrs::gettext;
#[allow(deprecated)]
use gtk::{
    prelude::FileChooserExt,
    prelude::{DialogExt, FileExt, GtkWindowExt},
    FileChooserAction, FileChooserDialog, ResponseType,
};
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::{error::CarteroError, win::CarteroWindow};

#[allow(deprecated)]
async fn handle_dialog_path(dialog: FileChooserDialog) -> Result<Option<PathBuf>, CarteroError> {
    let (tx, mut rx) = mpsc::channel::<Option<PathBuf>>(1);
    dialog.present();
    dialog.connect_response(move |dialog, res| {
        glib::spawn_future_local(glib::clone!(@strong dialog, @strong tx => async move {
            let path = if res == ResponseType::Accept {
                dialog.file().and_then(|f| f.path())
            } else {
                None
            };
            let _ = tx.send(path).await;
            dialog.destroy();
        }));
    });

    let path = rx.recv().await;
    if let Some(p) = path {
        Ok(p)
    } else {
        Err(CarteroError::FileDialogError)
    }
}

#[allow(deprecated)]
pub async fn open_file(win: &CarteroWindow) -> Result<Option<PathBuf>, CarteroError> {
    let dialog = FileChooserDialog::new(
        Some(gettext("Open request")),
        Some(win),
        FileChooserAction::Open,
        &[
            (&gettext("Open"), ResponseType::Accept),
            (&gettext("Cancel"), ResponseType::Cancel),
        ],
    );
    handle_dialog_path(dialog).await
}

#[allow(deprecated)]
pub async fn save_file(win: &CarteroWindow) -> Result<Option<PathBuf>, CarteroError> {
    let dialog = FileChooserDialog::new(
        Some(gettext("Save request")),
        Some(win),
        FileChooserAction::Save,
        &[
            (&gettext("Save"), ResponseType::Accept),
            (&gettext("Cancel"), ResponseType::Cancel),
        ],
    );
    handle_dialog_path(dialog).await
}
