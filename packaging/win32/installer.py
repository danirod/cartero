#!/usr/bin/env python3
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

# A stub to call ISCC.exe with the proper environment, given that ISCC.exe
# doesn't know about the environment variables defined in our environment.

import os
import sys
import subprocess

from argparse import ArgumentParser
from pathlib import Path

parser = ArgumentParser(description="Package Windows dependencies")
parser.add_argument(
    "-n",
    "--name",
    help="If given, will sign the application with the certificate owned by the given subject name",
)
parser.add_argument(
    "-i", "--installer", help="Path to the .iss installer script", required=True
)
args = parser.parse_args()


def sign(path: Path, subject: str):
    args1 = [
        "signtool",
        "sign",
        "/n",
        subject,
        "/t",
        "http://time.certum.pl",
        "/fd",
        "sha1",
        "/v",
        path,
    ]
    args2 = [
        "signtool",
        "sign",
        "/n",
        subject,
        "/tr",
        "http://time.certum.pl",
        "/fd",
        "sha256",
        "/td",
        "sha256",
        "/as",
        "/v",
        path,
    ]
    subprocess.run(args1)
    subprocess.run(args2)


install_dir = os.environ.get("MESON_INSTALL_DESTDIR_PREFIX")
install_args = ["iscc.exe"]
if install_dir:
    install_args += ["/DSOURCE_DIR=" + install_dir, "/O" + install_dir]
install_args += [args.installer]
subprocess.run(install_args)

if args.name:
    for installer in Path(install_dir).glob("*.exe"):
        sign(installer, args.name)
