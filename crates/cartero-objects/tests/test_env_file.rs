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

use std::{collections::HashMap, path::PathBuf};

use cartero_objects::{EnvFile, Field};
use gio::prelude::{FileExt, ListModelExt, ListModelExtManual};

#[test]
fn locate_for_path_same_directory() {
    let path = PathBuf::from("tests/env_files/existing/request.cartero");
    let file = gio::File::for_path(&path);

    let expected_env = gio::File::for_path(PathBuf::from("tests/env_files/existing/.env"));
    if !expected_env.query_exists(gio::Cancellable::NONE) {
        panic!(
            "Expected env file does not exist! {:?}",
            expected_env.path().unwrap()
        );
    }

    let Some(env_file) = EnvFile::locate_for_path(&file) else {
        panic!("Env file was not located!");
    };
    assert!(
        env_file.equal(&expected_env),
        "Got {:?}, expected {:?}",
        env_file.path().unwrap(),
        expected_env.path().unwrap()
    );
}

#[test]
fn locate_for_path_parent_directory() {
    let path = PathBuf::from("tests/env_files/existing/sub/request.cartero");
    let file = gio::File::for_path(&path);

    let expected_env = gio::File::for_path(PathBuf::from("tests/env_files/existing/.env"));
    if !expected_env.query_exists(gio::Cancellable::NONE) {
        panic!(
            "Expected env file does not exist! {:?}",
            expected_env.path().unwrap()
        );
    }

    let Some(env_file) = EnvFile::locate_for_path(&file) else {
        panic!("Env file was not located!");
    };
    assert!(
        env_file.equal(&expected_env),
        "Got {:?}, expected {:?}",
        env_file.path().unwrap(),
        expected_env.path().unwrap()
    );
}

#[test]
fn env_file_reads() {
    let env_file = gio::File::for_path(PathBuf::from("tests/env_files/existing/.env"));
    let env_data = EnvFile::builder().file(Some(&env_file)).build();
    assert!(env_data.file().is_some_and(|f| f.equal(&env_file)));
    assert_eq!(env_data.n_items(), 6);

    let mut variables = HashMap::new();
    env_data.iter::<Field>().for_each(|value| {
        if let Ok(field) = value {
            variables.insert(field.key(), field.value());
        }
    });

    assert_eq!(variables["API_KEY"], "12341234");
    assert_eq!(variables["WITH_SINGLE_QUOTES"], "value with single quotes");
    assert_eq!(variables["WITH_DOUBLE_QUOTES"], "value with double quotes");
    assert_eq!(variables["WITH_COMMENT_AFTER"], "after");
    assert_eq!(variables["EMPTY_VALUE"], "");
    assert_eq!(variables["WITH_SLASH"], "hac ker");
}

#[test]
fn env_file_reloads_on_path_change() {
    let env_file = gio::File::for_path(PathBuf::from("tests/env_files/existing/.env"));
    let env_data = EnvFile::builder().file(Some(&env_file)).build();
    assert!(env_data.file().is_some_and(|f| f.equal(&env_file)));
    assert_eq!(env_data.n_items(), 6);

    let new_env_file = gio::File::for_path(PathBuf::from("tests/env_files/.env"));
    env_data.set_file(Some(&new_env_file));
    assert!(env_data.file().is_some_and(|f| f.equal(&new_env_file)));
    assert_eq!(env_data.n_items(), 1);
}
