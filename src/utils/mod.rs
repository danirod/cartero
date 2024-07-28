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

use std::cell::RefCell;

use glib::property::PropertySet;
use gtk::ExpressionWatch;

#[derive(Default)]
pub struct SingleExpressionWatch(RefCell<Option<ExpressionWatch>>);

impl SingleExpressionWatch {
    pub fn replace(&self, watch: ExpressionWatch) {
        {
            let existing: &Option<ExpressionWatch> = &self.0.borrow();
            if let Some(existing) = existing {
                existing.unwatch();
            }
        }
        self.0.set(Some(watch));
    }

    pub fn clear(&self) {
        {
            let existing: &Option<ExpressionWatch> = &self.0.borrow();
            if let Some(existing) = existing {
                existing.unwatch();
            }
        }
        self.0.set(None);
    }
}
