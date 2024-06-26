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

use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};

use crate::{error::CarteroError, objects::Collection};

use super::metadata::Metadata;

pub fn save_collection(path: &Path, collection: &Collection) -> Result<(), CarteroError> {
    std::fs::create_dir(path)?;

    let metadata: Metadata = collection.into();
    let metadata_toml = toml::to_string(&metadata)?;
    let metadata_file = path.join("cartero-metadata");

    let mut file = File::create(metadata_file)?;
    write!(file, "{}", metadata_toml)?;
    Ok(())
}

pub fn open_collection(path: &Path) -> Result<Collection, CarteroError> {
    let metadata_file = path.join("cartero-metadata");

    // make sure that this is an actual collection
    if !metadata_file.exists() {
        return Err(CarteroError::NotValidCollection);
    }
    let metadata_content = std::fs::read_to_string(metadata_file)?;
    let metadata: Metadata = toml::from_str(&metadata_content)?;
    Ok(metadata.into())
}

fn is_cartero_request(entry: &Result<DirEntry, std::io::Error>) -> bool {
    let Ok(entry) = entry else {
        return false;
    };
    let path = entry.path();
    if !path.is_file() {
        return false;
    }
    let Some(ext) = path.as_path().extension() else {
        return false;
    };
    ext.to_str().is_some_and(|s| s == "cartero")
}

fn is_cartero_folder(entry: &Result<DirEntry, std::io::Error>) -> bool {
    let Ok(entry) = entry else {
        return false;
    };
    let path = entry.path();
    if !path.is_dir() {
        return false;
    }

    let metadata = path.join("cartero-metadata");
    metadata.exists() && metadata.is_file()
}

pub fn list_folders(path: &Path) -> Result<Vec<PathBuf>, CarteroError> {
    let folders = path
        .read_dir()?
        .filter(is_cartero_folder)
        .map(|entry| entry.unwrap().path())
        .collect();
    Ok(folders)
}

pub fn list_endpoints(path: &Path) -> Result<Vec<PathBuf>, CarteroError> {
    let endpoints = path
        .read_dir()?
        .filter(is_cartero_request)
        .map(|entry| entry.unwrap().path())
        .collect();
    Ok(endpoints)
}
