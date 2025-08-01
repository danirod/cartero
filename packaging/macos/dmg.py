#!/usr/bin/env python3
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

# This is the script that deals with signing the app.
#
# Note that this is not invoked by default when calling the install target of
# the meson script. You still have to call this manually, because there are
# some input parameters that cannot be bound to the Meson build script.

import os
import sys
import subprocess

from argparse import ArgumentParser
from pathlib import Path


def panic(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def var_lib(env_var):
    value = os.environ.get(env_var) or panic(f"{env_var} variable not set")
    path = Path(value)
    if not path.exists():
        panic(f"Directory {path} pointed by {env_var} does not exist")
    return path


def create_dmg(path: Path):
    self_dir = Path(__file__).parent
    args = [
        "create-dmg",
        "--volname",
        "Cartero",
        "--volicon",
        self_dir / "DmgIcon.icns",
        "--window-size",
        "854",
        "480",
        "--background",
        self_dir / "DmgBackground.png",
        "--icon-size",
        "96",
        "--icon",
        "Cartero.app",
        "210",
        "225",
        "--app-drop-link",
        "640",
        "225",
        "--eula",
        self_dir / ".." / "COPYING.rtf",
        path / "Cartero.dmg",
        path,
    ]
    subprocess.run(args)


def notarize(profile: str, path: Path):
    args = [
        "xcrun",
        "notarytool",
        "submit",
        path,
        "--keychain-profile",
        profile,
        "--wait",
    ]
    subprocess.run(args)


def staple(path: Path):
    args = ["xcrun", "stapler", "staple", path]
    subprocess.run(args)


destdir = var_lib("DESTDIR")

parser = ArgumentParser(description="Sign a macOS application")
parser.add_argument(
    "-p", "--profile", help="The keychain profile to use when notarizing"
)
args = parser.parse_args()

create_dmg(destdir)

if args.profile:
    notarize(args.profile, destdir / "Cartero.dmg")
    staple(destdir / "Cartero.dmg")
else:
    print("Keychain profile not given, skipping notarize step")
