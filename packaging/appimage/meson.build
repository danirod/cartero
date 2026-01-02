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

if host_machine.system() == 'linux' and get_option('appimage').enabled()
    desktop = '@0@.desktop'.format(application_id)
    install_symlink(
        desktop,
        install_dir: '/',
        pointing_to: datadir.strip('/') / 'applications' / desktop,
    )
    # NOTE: We don't actually do this because Cartero's SVG uses some fancy
    # filters that are rendered like shit when rsvg is not in use (for instance,
    # KDE). Uncomment this the day SVG rendering does not suck in KDE:
    # icon = '@0@.svg'.format(application_id)
    # install_symlink(icon, install_dir: '/', pointing_to: datadir.strip('/') / 'icons' / 'hicolor' / 'scalable' / 'apps' / icon)
    # install_symlink('.DirIcon', install_dir: '/', pointing_to: icon)
    meson.add_install_script('package.py', application_id)
endif
