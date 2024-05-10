#[allow(deprecated)]
use gtk4::{
    prelude::FileChooserExt,
    prelude::{DialogExt, FileExt, GtkWindowExt},
    FileChooserAction, FileChooserDialog, ResponseType,
};
use std::{error::Error, path::PathBuf};
use tokio::sync::mpsc;

use crate::win::CarteroWindow;

#[allow(deprecated)]
async fn handle_dialog_path(dialog: FileChooserDialog) -> Result<Option<PathBuf>, Box<dyn Error>> {
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
        Err("Something is wrong".into())
    }
}

#[allow(deprecated)]
pub async fn open_file(win: &CarteroWindow) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let dialog = FileChooserDialog::new(
        Some("Open request"),
        Some(win),
        FileChooserAction::Open,
        &[
            ("Accept", ResponseType::Accept),
            ("Cancel", ResponseType::Cancel),
        ],
    );
    handle_dialog_path(dialog).await
}

#[allow(deprecated)]
pub async fn save_file(win: &CarteroWindow) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let dialog = FileChooserDialog::new(
        Some("Save request"),
        Some(win),
        FileChooserAction::Save,
        &[
            ("Save", ResponseType::Accept),
            ("Cancel", ResponseType::Cancel),
        ],
    );
    handle_dialog_path(dialog).await
}
