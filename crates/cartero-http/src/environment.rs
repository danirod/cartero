// Copyright 2024-2026 the Cartero authors
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

use cartero_objects::EnvFile;

pub struct ClientConfig {
    pub validate_tls: bool,
    pub redirects: u64,
    pub timeout: f64,
}

pub struct ProxyConfig {
    pub respect_system_proxy: bool,
    pub http_proxy: String,
    pub https_proxy: String,
    pub no_proxy: Vec<String>,
}

pub struct RequestEnvironment {
    pub config: ClientConfig,
    pub proxy: Option<ProxyConfig>,
    pub prefix: Option<gio::File>,
    pub env_file: Option<EnvFile>,
}
