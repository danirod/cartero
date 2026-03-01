# Copyright 2024-2026 the Cartero authors
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

blueprint_files = [
    'authentication_pane.blp',
    'basic_auth_pane.blp',
    'bearer_token_pane.blp',
    'collapsed_field_table.blp',
    'endpoint_pane.blp',
    'error_pane.blp',
    'export_dialog.blp',
    'field_action_row.blp',
    'field_list_box_row.blp',
    'field_list_box_static_row.blp',
    'field_placeholder_row.blp',
    'field_table_list_view.blp',
    'file_body_pane.blp',
    'mac_menu.blp',
    'main_window_no_csd.blp',
    'main_window.blp',
    'method_dropdown.blp',
    'multipart_body_pane.blp',
    'raw_body_pane.blp',
    'request_body_pane.blp',
    'response_headers.blp',
    'response_panel.blp',
    'search_box.blp',
    'settings/page_appearance.blp',
    'settings/page_application.blp',
    'settings/page_code_editor.blp',
    'settings/page_http_client.blp',
    'settings/page_proxy.blp',
    'settings/page_security.blp',
    'settings/pill.blp',
    'settings/shell.blp',
    'urlencoded_body_pane.blp',
    'welcome/welcome_pane.blp',
]

blueprint_targets = []
foreach blp : blueprint_files
    blueprint_targets += blp.replace('.blp', '.ui').split('/')[-1]
endforeach

blueprints = custom_target(
    'blueprints',
    input: files(blueprint_files),
    output: blueprint_targets,
    command: [
        find_program('blueprint-compiler'),
        'batch-compile',
        '@OUTDIR@',
        '@CURRENT_SOURCE_DIR@',
        '@INPUT@',
    ],
)
