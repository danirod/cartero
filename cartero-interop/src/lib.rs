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

use std::error::Error;

type InnerError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct FileLoadResult<T>
where
    T: Clone,
{
    object: T,
    warnings: Vec<FileWarningTag>,
}

impl<T> FileLoadResult<T>
where
    T: Clone,
{
    pub fn new(obj: T, warnings: &[FileWarningTag]) -> Self {
        Self {
            object: obj.clone(),
            warnings: warnings.to_vec(),
        }
    }

    pub fn object(&self) -> &T {
        &self.object
    }

    pub fn warnings(&self) -> Vec<FileWarningTag> {
        self.warnings.clone()
    }
}

/// A recoverable error, like a linter error. The file will continue to be
/// opened, and the user interface will be updated properly, but the user
/// should know that something is not right and that the data should be
/// checked.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FileWarningTag {
    /// A request file had an inappropiate HTTP verb, and it has been reset to the default one.
    InvalidHttpVerb(String),
}

/// An unrecoverable error, like a parse error. The file cannot be opened due
/// to a critical condition. Given that the format is plain text, the user may
/// be able to open the affected file to recover data.
#[derive(Debug)]
pub enum FileLoadError {
    /// The file is using a schema that is too recent for this version of the app.
    SchemaTooNew,
    /// There is a parse error or something that prevents data from being decoded.
    DeserializationError(InnerError),
}

/// An unrecoverable error that prevents data from being saved. It should be
/// assumed that if this error is thrown in the program, then the data might
/// not actually have been saved.
pub enum FileSaveError {
    SerializationError(InnerError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_file_load_result() {
        let result = FileLoadResult {
            object: "foobar".to_string(),
            warnings: vec![FileWarningTag::InvalidHttpVerb("DELETE".into())],
        };
        assert_eq!("foobar".to_string(), *result.object());

        let outcome = result.warnings();
        assert_eq!(1, outcome.len());
        assert_eq!(FileWarningTag::InvalidHttpVerb("DELETE".into()), outcome[0]);
    }
}
