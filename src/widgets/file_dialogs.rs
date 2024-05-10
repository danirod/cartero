use gtk4::prelude::FileExt;
use std::{error::Error, path::PathBuf};

use crate::win::CarteroWindow;

pub async fn open_file(win: &CarteroWindow) -> Result<PathBuf, Box<dyn Error>> {
    let dialog = gtk4::FileDialog::builder()
        .accept_label("Abrir")
        .title("Abrir petición")
        .modal(true)
        .build();
    let result = dialog.open_future(Some(win)).await;
    let file = result.map_err(Box::new)?;
    let path = file.path().ok_or("No path")?;
    Ok(path)
}

pub async fn save_file(win: &CarteroWindow) -> Result<PathBuf, Box<dyn Error>> {
    let dialog = gtk4::FileDialog::builder()
        .accept_label("Guardar")
        .title("Guardar petición")
        .modal(true)
        .build();
    let result = dialog.save_future(Some(win)).await;
    let file = result.map_err(Box::new)?;
    let path = file.path().ok_or("No path")?;
    Ok(path)
}
