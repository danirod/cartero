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

# Environment variables provided by Meson (if you are not using Meson, beware,
# you have to give these manually and they have to work!):
#
# - MESON_INSTALL_PREFIX: will be appended to DESTDIR. This is the location
#   where the distribution is being installed. Usually DESTDIR is set to
#   somewhere close to the source code being built, and the prefix is set to
#   /usr or /usr/local.
# - MESON_INSTALL_DESTDIR_PREFIX: DESTDIR + MESON_INSTALL_PREFIX.


import os
import re
import shutil
import sys
import subprocess

from pathlib import Path


def panic(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def pkg_config(package, variable):
    args = ["pkg-config", "--variable=" + variable, package]
    output = subprocess.check_output(args).decode("utf-8")
    return output.strip()


def var_lib(env_var):
    value = os.environ.get(env_var) or panic(f"{env_var} variable not set")
    path = Path(value)
    if not path.exists():
        panic(f"Directory {path} pointed by {env_var} does not exist")
    return path


def shared_library_is_standard(path):
    # These are protected directories in macOS, so if a library is here, it is
    # part of the system, it hasn't been added by a program.
    return path.startswith("/usr/lib") or path.startswith("/System/Library")


def shared_libraries(path):
    args = ["otool", "-L", path]
    output = subprocess.check_output(args).decode("utf-8")
    otool_lib = r"\t(.*) \(.*\)"
    return re.findall(otool_lib, output)


def force_sign_file(path):
    args = [
        "codesign",
        "--sign",
        "-",
        "--force",
        "--preserve-metadata=entitlements,requirements,flags,runtime",
        path,
    ]
    subprocess.run(args)


def relink_dependency(path, old, new):
    """Wraps a call to install_name_tool"""
    args = ["install_name_tool", "-change", old, new, path]
    print("relinking:", args)
    subprocess.run(args)
    force_sign_file(path)


# Environment variables for main paths
destdir = var_lib("DESTDIR")
source_dir = var_lib("MESON_SOURCE_ROOT")
root_dir = var_lib("MESON_INSTALL_PREFIX")
install_dir = var_lib("MESON_INSTALL_DESTDIR_PREFIX")

# Main system paths
bindir = install_dir / "bin"
libdir = install_dir / "lib"
datadir = install_dir / "share"


def gettext_linguas():
    linguas_file = source_dir / "po" / "LINGUAS"
    linguas_data = linguas_file.read_text().splitlines()
    return [l for l in linguas_data if not l.startswith("#")]


def relocate_and_vendor(path, relative_linker_path_to_lib, rpath=[]):
    print(f"Relocating {path}...")
    relative_linker_path = Path(relative_linker_path_to_lib)
    third_party_deps = [
        Path(dep)
        for dep in shared_libraries(path)
        if not shared_library_is_standard(dep)
    ]
    for dep_path in third_party_deps:
        real_dep_path = dep_path
        if str(dep_path).startswith("@rpath"):
            # Unmangle rpath by looking for the first lib in rpath that exists.
            rpath_candidates = [
                Path(str(dep_path).replace("@rpath", str(r))).resolve() for r in rpath
            ]
            dep_path = next((p for p in rpath_candidates if p.exists()))

        new_dep_path = relative_linker_path / dep_path.name
        relink_dependency(path, real_dep_path, new_dep_path)

        target_path = libdir / dep_path.name
        if not target_path.exists():
            print(f"Vendoring {dep_path} into {target_path}...")
            if not libdir.exists():
                libdir.mkdir()
            shutil.copy(dep_path, target_path)
            relocate_and_vendor(target_path, "@loader_path", rpath)


# Get the application ID from the argv.
# TODO: This can be inferred from the metainfo XML file.
if len(sys.argv) < 2:
    print(f"Missing arguments: {sys.argv[0]} [app_id]")
    sys.exit(1)
_, app_id = sys.argv

# Start relocating.
relocate_and_vendor(bindir / "cartero", "@loader_path/../lib")

# Bring the gdk-pixbuf-2.0 loaders too
pixbuf_bindir = libdir / "gdk-pixbuf-2.0" / "2.10.0"
pixbuf_moduledir = pixbuf_bindir / "loaders"
pixbuf_moduledir.mkdir(exist_ok=True, parents=True)

# The loaders will need to be relocated as well.
gdk_pixbuf_rpath = [
    Path(pkg_config("gdk-pixbuf-2.0", "libdir")),
    Path(pkg_config("librsvg-2.0", "libdir")),
]
pixbuf_moduledir_src = Path(pkg_config("gdk-pixbuf-2.0", "gdk_pixbuf_moduledir"))
for module in pixbuf_moduledir_src.glob("*.so"):
    target_path = pixbuf_moduledir / module.name
    if target_path.exists():
        target_path.unlink()
    shutil.copy(module, target_path)
    relocate_and_vendor(target_path, "@loader_path/../../..", rpath=gdk_pixbuf_rpath)

# Generate a new loaders.cache file
query_loaders = pkg_config("gdk-pixbuf-2.0", "gdk_pixbuf_query_loaders")
loaders_cache = subprocess.check_output(
    [query_loaders],
    env={
        "GDK_PIXBUF_MODULEDIR": pixbuf_moduledir,
    },
).decode("utf-8")
loaders_cache = loaders_cache.replace(
    str(pixbuf_moduledir) + "/", "@loader_path/gdk-pixbuf-2.0/2.10.0/loaders/"
)
with open(pixbuf_bindir / "loaders.cache", mode="w") as file:
    file.write(loaders_cache)

# Vendor icon theme
adwaita_icon_theme_pc = Path(pkg_config("adwaita-icon-theme", "pcfiledir"))
adwaita_icons_src = adwaita_icon_theme_pc / ".." / "icons" / "Adwaita"
adwaita_icons = datadir / "icons" / "Adwaita"
shutil.copytree(adwaita_icons_src, adwaita_icons, dirs_exist_ok=True)
subprocess.run(
    ["gtk4-update-icon-cache", "-q", "-t", "-f", datadir / "icons" / "hicolor"]
)

# Vendor GtkSource data files
gtksource_root = Path(pkg_config("gtksourceview-5", "prefix"))
gtksource_src = gtksource_root / "share" / "gtksourceview-5"
gtksource = datadir / "gtksourceview-5"
shutil.copytree(gtksource_src, gtksource, dirs_exist_ok=True)

# Vendor GTK schemas
glib_root = Path(pkg_config("glib-2.0", "prefix"))
glib_schemas_src = glib_root / "share" / "glib-2.0" / "schemas"
glib_schemas = datadir / "glib-2.0" / "schemas"
for root, _, files in glib_schemas_src.walk():
    for file in files:
        if file.startswith("org.gtk."):
            src_file = root / file
            dest_file = glib_schemas / file
            shutil.copy(src_file, dest_file)

gtk4_root = Path(pkg_config("gtk4", "prefix"))
gtk4_schemas_src = gtk4_root / "share" / "glib-2.0" / "schemas"
glib_schemas = datadir / "glib-2.0" / "schemas"
for root, _, files in gtk4_schemas_src.walk():
    for file in files:
        if file.startswith("org.gtk."):
            src_file = root / file
            dest_file = glib_schemas / file
            shutil.copy(src_file, dest_file)
subprocess.run(["glib-compile-schemas", datadir / "glib-2.0" / "schemas"])

# Vendor additional gettext packages
packages = [
    "gdk-pixbuf-2.0",
    "glib-2.0",
    "gtk4",
    "gtksourceview-5",
    "libadwaita-1",
    "shared-mime-info",
]
linguas = gettext_linguas()
for package in packages:
    package_root = pkg_config(package, "prefix")
    for lang in linguas:
        locale_root = Path(package_root) / "share" / "locale" / lang / "LC_MESSAGES"
        if locale_root.exists():
            target_dir = datadir / "locale" / lang / "LC_MESSAGES"
            target_dir.mkdir(exist_ok=True)
            for locale_file in locale_root.glob("*.mo"):
                shutil.copy(locale_file, target_dir / locale_file.name)
