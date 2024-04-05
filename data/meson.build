# Copyright 2024 the Cartero authors
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# SPDX-License-Identifier: GPL-3.0-or-later

subdir('icons')

blueprint_files = [
  'ui/row_header.blp',
  'ui/main_window.blp',
  'ui/response_panel.blp',
]

blueprints = custom_target('blueprints',
  input: files(blueprint_files),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

resources = gnome.compile_resources(
  'cartero',
  'cartero.gresource.xml',
  dependencies: blueprints,
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
)

desktop_file = configure_file(
  input: 'cartero.desktop.in',
  output: '@0@.desktop'.format(application_id),
  configuration: { 'app_id': application_id },
  install_dir: join_paths(get_option('datadir'), 'applications')
)
desktop_utils = find_program('desktop-file-validate', required: true)
test('Validate desktop file', desktop_utils, args: [desktop_file])
