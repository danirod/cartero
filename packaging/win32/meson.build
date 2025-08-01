# Copyright 2024-2025 the Cartero authors
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

if host_machine.system() == 'windows'
    sign_subject = get_option('win32-sign-subject').strip()

    if get_option('win32-bundle').enabled() or get_option('win32-installer').enabled()
        if sign_subject != ''
            meson.add_install_script('dependencies.py', '-n', sign_subject)
        else
            meson.add_install_script('dependencies.py')
        endif
    endif

    if get_option('win32-installer').enabled()
        windows_setup = configure_file(
            input: 'installer.iss.in',
            output: 'installer.iss',
            configuration: {
                'MESON_SOURCE_ROOT': meson.project_source_root(),
                'VERSION': version,
            },
        )

        if sign_subject != ''
            meson.add_install_script(
                'installer.py',
                '-i',
                windows_setup,
                '-n',
                sign_subject,
            )
        else
            meson.add_install_script('installer.py', '-i', windows_setup)
        endif
    endif
endif
