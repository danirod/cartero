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

use glib::{object::ObjectBuilder, Object};
use gtk::prelude::{FileExtManual, SettingsExtManual};

use crate::{
    app::CarteroApplication,
    file::{CarteroMetadata, FileError},
};

use super::{persistable::PersistableExt, Persistable};

mod imp {
    use std::cell::RefCell;

    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use glib::Properties;

    use crate::objects::KeyValueStore;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Collection)]
    pub struct Collection {
        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set)]
        description: RefCell<String>,

        #[property(get)]
        variables: RefCell<KeyValueStore>,

        #[property(get, set, nullable)]
        path: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Collection {
        const NAME: &'static str = "CarteroCollection";
        type Type = super::Collection;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Collection {}
}

glib::wrapper! {
    pub struct Collection(ObjectSubclass<imp::Collection>)
        @implements gtk::Buildable, Persistable;
}

impl Collection {
    pub fn builder() -> ObjectBuilder<'static, Self> {
        Object::builder()
    }
}

impl Default for Collection {
    fn default() -> Self {
        Collection::builder().build()
    }
}

impl PersistableExt for Collection {
    async fn load_file(&self) -> Result<(), FileError> {
        let Some(file) = self.path().map(gtk::gio::File::for_path) else {
            return Err(FileError::NoPath);
        };
        let contents = file
            .load_contents_future()
            .await
            .map(|data| String::from_utf8_lossy(&data.0).to_string())
            .map_err(|_| FileError::CannotRead)?;
        let metadata =
            toml::from_str::<CarteroMetadata>(&contents).map_err(|_| FileError::CannotParseFile)?;
        metadata.assign_properties(self);
        Ok(())
    }

    async fn save_file(&self) -> Result<(), FileError> {
        let Some(file) = self.path().map(gtk::gio::File::for_path) else {
            return Err(FileError::NoPath);
        };
        let metadata = CarteroMetadata::from(self);
        let contents = toml::to_string(&metadata).map_err(|_| FileError::CannotEncodePayload)?;

        let app = CarteroApplication::default();
        let settings = app.settings();
        let use_backups = settings.get::<bool>("create-backup-files");
        file.replace_contents_future(
            contents.to_string(),
            None,
            use_backups,
            gtk::gio::FileCreateFlags::NONE,
        )
        .await
        .map_err(|_| FileError::CannotSave)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use glib::object::Cast;
    use gtk::prelude::ListModelExt;

    use crate::{
        file::FileError,
        objects::{persistable::PersistableExt, KeyValueItem},
    };

    use super::Collection;

    #[test]
    pub fn test_collection_builder() {
        let collection = Collection::builder()
            .property("title", "PokeAPI")
            .property("description", "This is the PokeAPI collection")
            .build();

        assert_eq!(collection.title(), "PokeAPI");
        assert_eq!(collection.description(), "This is the PokeAPI collection");
        assert_eq!(collection.variables().n_items(), 0);
    }

    #[gtk::test]
    pub async fn test_collection_without_file() {
        let collection = Collection::default();
        let result = collection.load_file().await;
        assert!(result.is_err_and(|e| e == FileError::NoPath));
    }

    #[gtk::test]
    pub async fn test_collection_with_file() {
        let collection = Collection::default();
        let root = env!("CARGO_MANIFEST_DIR");
        let path = format!("{root}/tests/mocks/collection-metadata.toml");
        collection.set_path(Some(path));
        let result = collection.load_file().await;
        assert!(result.is_ok());
        assert_eq!(collection.title(), "Meow Facts");
        assert_eq!(
            collection.description(),
            "The public API to get data about cats"
        );

        let variables = collection.variables();
        assert_eq!(2, variables.n_items());

        {
            let kv = variables
                .item(0)
                .unwrap()
                .downcast::<KeyValueItem>()
                .unwrap();
            assert_eq!("API_ROOT", kv.header_name());
            assert_eq!("meowfacts.herokuapp.com", kv.header_value());
            assert!(kv.active());
            assert!(!kv.secret());
        }
        {
            let kv: KeyValueItem = variables
                .item(1)
                .unwrap()
                .downcast::<KeyValueItem>()
                .unwrap();
            assert_eq!("FAKE_API_KEY", kv.header_name());
            assert_eq!("1234", kv.header_value());
            assert!(!kv.active());
            assert!(kv.secret());
        }
    }

    #[gtk::test]
    pub async fn test_collection_load_resets_variables() {
        let collection = Collection::default();
        let key = KeyValueItem::builder()
            .property("header-name", "API_KEY")
            .property("header-value", "123412341234")
            .property("active", false)
            .property("secret", true)
            .build();
        collection.variables().insert(&key);
        assert_eq!(1, collection.variables().n_items());
        let root = env!("CARGO_MANIFEST_DIR");
        let path = format!("{root}/tests/mocks/collection-metadata-no-variables.toml");
        collection.set_path(Some(path));
        let result = collection.load_file().await;
        assert!(result.is_ok());
        assert_eq!(0, collection.variables().n_items());
    }

    #[gtk::test]
    pub async fn test_collection_save_fails_when_no_path() {
        let collection: Collection = glib::Object::builder()
            .property("title", "PokeAPI")
            .property("description", "This is the PokeAPI collection")
            .build();
        let key = KeyValueItem::builder()
            .property("header-name", "API_KEY")
            .property("header-value", "123412341234")
            .property("active", false)
            .property("secret", true)
            .build();
        collection.variables().insert(&key);

        let result = collection.save_file().await;
        assert!(result.is_err_and(|e| e == FileError::NoPath));
    }

    #[gtk::test]
    pub async fn test_collection_save_when_path() {
        let collection: Collection = glib::Object::builder()
            .property("title", "PokeAPI")
            .property("description", "This is the PokeAPI collection")
            .build();
        let key = KeyValueItem::builder()
            .property("header-name", "API_KEY")
            .property("header-value", "123412341234")
            .property("active", false)
            .property("secret", true)
            .build();
        let key2 = KeyValueItem::builder()
            .property("header-name", "PROTOCOL")
            .property("header-value", "https")
            .property("active", true)
            .property("secret", false)
            .build();
        collection.variables().insert(&key);
        collection.variables().insert(&key2);

        let tmp_dir = mktemp::Temp::new_file().unwrap();
        let tmp_path = String::from(tmp_dir.to_string_lossy());
        collection.set_path(Some(tmp_path));

        let result = collection.save_file().await;
        assert!(result.is_ok());

        let result = std::fs::read(&tmp_dir);
        assert!(result.is_ok());
        if let Ok(contents) = result {
            let str_contents = String::from_utf8_lossy(&contents);
            assert!(str_contents.contains("title = \"PokeAPI\""));
            assert!(str_contents.contains("description = \"This is the PokeAPI collection\""));
            assert!(str_contents.contains("PROTOCOL = \"https\""));
        }
    }
}
