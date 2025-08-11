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

import os
import re
import sys
import shutil
import subprocess

from argparse import ArgumentParser
from pathlib import Path, PureWindowsPath

# This script bundles the required dependencies to use Cartero in Windows
# after compiling it in an MSYS environment. The .dlls and other datafiles
# will be present in $MSYS_PREFIX, but we need to copy those to the
# installation directory to make them available when running the application
# if MSYS2 is not installed.

LDD_REGEX = re.compile(r"\s*(.+) => (.+) \(0x[a-z0-9]+\)")

parser = ArgumentParser(description="Package Windows dependencies")
parser.add_argument(
    "-n",
    "--name",
    help="If given, will sign the application with the certificate owned by the given subject name",
)
args = parser.parse_args()


def cygpath(path):
    path = subprocess.run(["cygpath", path], stdout=subprocess.PIPE).stdout
    path = path.decode("utf-8").strip()
    return PureWindowsPath(path)


def winpath(path):
    path = subprocess.run(["cygpath", "-w", path], stdout=subprocess.PIPE).stdout
    path = path.decode("utf-8").strip()
    return Path(path)


def var_lib(env_var):
    value = os.environ.get(env_var)
    if not value:
        return None
    path = Path(value)
    if not path.exists():
        return None
    return path


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


msys = var_lib("MINGW_PREFIX")
if msys is None:
    print("MINGW_PREFIX environment variable not set or directory missing!")
    sys.exit(1)

install_dir = var_lib("MESON_INSTALL_DESTDIR_PREFIX")
if install_dir is None:
    print("MESON_INSTALL_DESTDIR_PREFIX variable not set or directory missing!")
    sys.exit(1)

source_dir = var_lib("MESON_SOURCE_ROOT")
if source_dir is None:
    print("MESON_SOURCE_ROOT variable not set or directory missing!")
    sys.exit(1)


bindir = install_dir / "bin"
libdir = install_dir / "lib"
datadir = install_dir / "share"


def find_deps(file):
    if isinstance(file, Path):
        file = str(file.absolute())
    output = subprocess.run(["ldd.exe", file], stdout=subprocess.PIPE, cwd=bindir)
    lines = output.stdout.decode("utf-8")
    deps = {}
    cygroot = cygpath(msys)
    for match in LDD_REGEX.finditer(lines):
        orig = Path(match.group(2))
        if cygroot in orig.parents:
            deps[match.group(1)] = winpath(orig)
    return deps


# Binary dependencies that have to be copied to bindir
bindeps = ["gdbus.exe", "gspawn-win64-helper.exe", "vulkan-1.dll"]
for dep in bindeps:
    print(f"  Copying {dep} to {bindir / dep}...")
    shutil.copy(msys / "bin" / dep, bindir / dep)

# Copy DLL dependencies for cartero.exe
deps = find_deps(bindir / "cartero.exe")
for name, path in deps.items():
    print(f"Copying {path} to {bindir / name}...")
    shutil.copy(path, bindir / name)

# Copy gdk-pixbuf-2.0 loaders.
shutil.copytree(
    msys / "lib" / "gdk-pixbuf-2.0", libdir / "gdk-pixbuf-2.0", dirs_exist_ok=True
)

# Copy dependencies for gdk-pixbuf-2.0 loaders.
loaders = libdir / "gdk-pixbuf-2.0" / "2.10.0" / "loaders"
for root, _, files in loaders.walk():
    files = [f for f in files if f.endswith(".dll")]  # ignore .a files
    for file in files:
        deps = find_deps(root / file)
        for name, path in deps.items():
            print(f"Copying {path} to {bindir / name}...")
            shutil.copy(path, bindir / name)

# For some reason, sometimes librsvg is not copied, at least in CLANGARM64.
# TODO: Why? The reason is not clear. Running ldd.exe in CLANGARM64 does not
# initially reveal that librsvg is required. However, when running ldd in
# another clean CLANGARM64 root shows the librsvg dependency.
librsvg = next((msys / "bin").glob("librsvg*.dll"))
if not librsvg:
    print("Error: librsvg DLL not found!")
    sys.exit(1)
shutil.copy(librsvg, bindir / librsvg.name)

# Copy datafiles required by dependencies
datadeps = ["glib-2.0", "gtksourceview-5", "icons/Adwaita", "icons/hicolor"]
for dep in datadeps:
    shutil.copytree(msys / "share" / dep, datadir / dep, dirs_exist_ok=True)

# Copy gettext packages
linguas_file = source_dir / "po" / "LINGUAS"
linguas = [l.strip() for l in open(linguas_file).readlines() if not l.startswith("#")]
gettext_packages = [
    "gdk-pixbuf",
    "gettext-runtime",
    "glib20",
    "gtk40",
    "gtksourceview-5",
    "libadwaita",
    "shared-mime-info",
]
for lang in linguas:
    for pkg in gettext_packages:
        mo_file = msys / "share" / "locale" / lang / "LC_MESSAGES" / f"{pkg}.mo"
        if mo_file.exists():
            print(f"Copying .mo file for {pkg} ({lang})...")
            shutil.copy(
                mo_file, datadir / "locale" / lang / "LC_MESSAGES" / f"{pkg}.mo"
            )
        else:
            print(f".mo file for {pkg} not found for locale {lang}, skipping...")

# Post-install GTK actions...
subprocess.run(["glib-compile-schemas.exe", str(datadir / "glib-2.0" / "schemas")])
subprocess.run(["gtk4-update-icon-cache.exe", str(datadir / "icons" / "hicolor")])

# Sign the application if a signature has been given
if args.name:
    sign(bindir / "cartero.exe", args.name)
