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
  'ui/endpoint_pane.blp',
  'ui/main_window.blp',
  'ui/main_window_no_csd.blp',
  'ui/key_value_pane.blp',
  'ui/key_value_row.blp',
  'ui/payload_tab.blp',
  'ui/raw_payload_pane.blp',
  'ui/response_headers.blp',
  'ui/response_panel.blp',
  'ui/save_dialog.blp',
  'ui/urlencoded_payload_pane.blp',
]

blueprint_targets = []
foreach blp : blueprint_files
  blueprint_targets += blp.replace('.blp', '.ui').split('/')[-1]
endforeach

blueprints = custom_target('blueprints',
  input: files(blueprint_files),
  output: blueprint_targets,
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTDIR@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
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

install_data('es.danirod.Cartero.gschema.xml', install_dir: datadir / 'glib-2.0' / 'schemas')
glib_compile_schemas = find_program('glib-compile-schemas', required: false)
if glib_compile_schemas.found()
  test('Validate schema file', glib_compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif


desktop_file = configure_file(
  input: 'cartero.desktop.in',
  output: '@0@.desktop'.format(application_id),
  configuration: { 'app_id': application_id },
  install_dir: join_paths(get_option('datadir'), 'applications')
)
desktop_utils = find_program('desktop-file-validate', required: true)
test('Validate desktop file', desktop_utils, args: [desktop_file])
