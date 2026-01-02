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

if host_machine.system() == 'darwin'
    if get_option('macos-app').enabled() or get_option('macos-dmg').enabled()
        if host_machine.cpu_family() == 'aarch64'
            info_plist_simplified_arch = 'arm64'
        elif host_machine.cpu_family() == 'x86_64'
            info_plist_simplified_arch = 'x86_64'
        else
            error('Unsupported host architecture')
        endif

        configure_file(
            input: 'Info.plist.in',
            output: 'Info.plist',
            configuration: {
                'BUNDLE_ID': application_id,
                'APP_VERSION': version,
                'ARCH': info_plist_simplified_arch,
            },
        )

        meson.add_install_script('vendoring.py', application_id)
        meson.add_install_script(
            'packaging.py',
            '-i',
            get_option('macos-codesign-identity'),
        )
    endif

    if get_option('macos-dmg').enabled()
        meson.add_install_script(
            'dmg.py',
            '-p',
            get_option('macos-notary-profile'),
        )
    endif
endif
