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

use glib::subclass::prelude::*;

glib::wrapper! {
    pub struct RequestAuthenticationData(ObjectSubclass<imp::RequestAuthenticationData>);
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct RequestAuthenticationData;

    #[glib::object_subclass]
    impl ObjectSubclass for RequestAuthenticationData {
        const NAME: &'static str = "CarteroRequestAuthorizationData";
        type Type = super::RequestAuthenticationData;
    }

    impl ObjectImpl for RequestAuthenticationData {}
}

pub trait RequestAuthenticationDataImpl: ObjectImpl {}

unsafe impl<T: RequestAuthenticationDataImpl> IsSubclassable<T> for RequestAuthenticationData {}
