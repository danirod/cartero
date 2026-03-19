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

# A little trick to keep Cartero from compiling on Debian 13 and other systems
# without libadwaita 1.8, while at the same time using the newer keyboard
# shortcuts window for Adwaita 1.8 everywhere else.
if adwaita.version().version_compare('>= 1.8')
  help_overlay = custom_target(
    'blueprint-help-overlay',
    input: files('adw_help_overlay.blp'),
    output: 'help_overlay.ui',
    command: [
      find_program('blueprint-compiler'),
      'compile',
      '--output',
      '@OUTDIR@/help_overlay.ui',
      '@INPUT@',
    ]
  )
else
  help_overlay = custom_target(
    'blueprint-help-overlay',
    input: files('gtk_help_overlay.blp'),
    output: 'help_overlay.ui',
    command: [
      find_program('blueprint-compiler'),
      'compile',
      '--output',
      '@OUTDIR@/help_overlay.ui',
      '@INPUT@',
    ]
  )
endif
