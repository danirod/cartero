#!/usr/bin/env python3

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
import sys
import shutil
import subprocess

from pathlib import Path


def var_lib(env_var):
    value = os.environ.get(env_var)
    if not value:
        print(f"{env_var} variable not set")
        sys.exit(1)
    path = Path(value)
    if not path.exists():
        print(f"Directory {path} pointed by {env_var} does not exist")
        sys.exit(1)
    return path


def which(app):
    path = shutil.which(app)
    if not path:
        print(f"Executable {app} not found")
        sys.exit(1)
    return Path(path)


# Required to copy files such as AppRun
template_path = Path(__file__).parent

# Extract environment variables
destdir = var_lib("DESTDIR")
root_dir = var_lib("MESON_INSTALL_PREFIX")
install_dir = var_lib("MESON_INSTALL_DESTDIR_PREFIX")

# Common directories for the installer
bindir = install_dir / "bin"
libdir = install_dir / "lib"
datadir = install_dir / "share"

# patchelf will take care of relocating executables and libraries
patchelf = which("patchelf")

# Guess where GTK is installed using the location of one of its dev binaries
# Yes, we use resolve() because on some systems, /sbin is a symlink to /usr/bin
gtk_root = which("gtk4-update-icon-cache").parent.resolve().parent

# Some information has to be infered from the argv.
if len(sys.argv) < 2:
    print(f"Missing arguments: {sys.argv[0]} [app_id]")
    sys.exit(1)
_, app_id = sys.argv


def patchelf_needed(path):
    """Wraps a call to patchelf --print-needed"""
    args = [patchelf, "--print-needed", path]
    cmd = subprocess.run(args, stdout=subprocess.PIPE)
    stdout = cmd.stdout.decode("utf-8").strip()
    return stdout.splitlines()


def patchelf_set_rpath(path, rpath):
    """Wraps a call to patchelf --set-rpath"""
    print(f"Setting rpath to {rpath} for {path}...")
    subprocess.run([patchelf, "--set-rpath", rpath, path], capture_output=True)


def patchelf_replace_needed(path, old_lib, new_lib):
    print(f"Replacing needed from {old_lib} to {new_lib} for {path}")
    args = [patchelf, "--replace-needed", old_lib, new_lib, path]
    subprocess.run(args, capture_output=True)


def copy_shared_libraries(path):
    new_libs = set()
    if not libdir.exists():
        libdir.mkdir()
    for dep_name in patchelf_needed(path):
        dep_file = (gtk_root / "lib" / dep_name).resolve()
        target_loc = libdir / dep_name
        if not target_loc.exists():
            print(f"Vendoring {dep_file} to {target_loc} as a dependency...")
            shutil.copy(dep_file, target_loc)
            patchelf_set_rpath(target_loc, "$ORIGIN")
            patchelf_replace_needed(target_loc, dep_name, dep_name)
            new_libs.add(dep_name)
    return new_libs


# Relocate usr/bin/cartero
patchelf_set_rpath(bindir / "cartero", "$ORIGIN/../lib")
new_deps = copy_shared_libraries(bindir / "cartero")
while len(new_deps) > 0:
    next_deps = set()
    for new_dep in new_deps:
        new_dep_path = libdir / new_dep
        next_deps = next_deps.union(copy_shared_libraries(new_dep_path))
    new_deps = next_deps

# Copy gdk-pixbuf-2.0 loaders
gdk_pixbuf = libdir / "gdk-pixbuf-2.0" / "2.10.0" / "loaders"
gdk_pixbuf.mkdir(exist_ok=True, parents=True)
gdk_pixbuf_src = gtk_root / "lib" / "gdk-pixbuf-2.0" / "2.10.0" / "loaders"
shutil.copytree(gdk_pixbuf_src, gdk_pixbuf, dirs_exist_ok=True)

# Recompile loaders
gdk_pixbuf_loaders = subprocess.run(
    ["gdk-pixbuf-query-loaders"],
    env={"GDK_PIXBUF_MODULEDIR": str(gdk_pixbuf)},
    stdout=subprocess.PIPE,
)
gdk_pixbuf_loaders = gdk_pixbuf_loaders.stdout.decode("utf-8")
gdk_pixbuf_loaders = re.sub(
    r".*/lib/gdk-pixbuf-2.0", r'"gdk-pixbuf-2.0', gdk_pixbuf_loaders
)
gdk_pixbuf_loader_cache = libdir / "gdk-pixbuf-2.0" / "2.10.0" / "loaders.cache"
with open(gdk_pixbuf_loader_cache, mode="w") as file:
    file.write(gdk_pixbuf_loaders)

# Then relocate loaders too
pixbuf_deps = set()
for pixbuf_loader in gdk_pixbuf.glob("*.so"):
    patchelf_set_rpath(pixbuf_loader, "$ORIGIN/../../../")
    pixbuf_deps = pixbuf_deps.union(copy_shared_libraries(pixbuf_loader))
while len(pixbuf_deps) > 0:
    next_deps = set()
    for pixbuf_dep in pixbuf_deps:
        pixbuf_dep_path = libdir / pixbuf_dep
        next_deps = next_deps.union(copy_shared_libraries(pixbuf_dep_path))
    pixbuf_deps = next_deps


# Copy AppRun script
shutil.copy(template_path / "AppRun", destdir / "AppRun")

# Create AppImage icons
icon_root = datadir / "icons" / "hicolor"
if rsvg_convert := shutil.which("rsvg-convert"):
    icon_svg = icon_root / "scalable" / "apps" / f"{app_id}.svg"
    icon_sizes = [16, 24, 32, 48, 64, 96, 128, 256, 512]
    for icon_size in icon_sizes:
        icon_dir = icon_root / f"{icon_size}x{icon_size}" / "apps"
        icon_dir.mkdir(parents=True, exist_ok=True)
        icon_file = icon_dir / f"{app_id}.png"
        subprocess.run(
            [
                rsvg_convert,
                "-w",
                str(icon_size),
                "-h",
                str(icon_size),
                "-f",
                "png",
                "-o",
                icon_file,
                icon_svg,
            ],
        )

    # Symlink the 512x512 version to be used as the AppImage icon.
    hd_icon = icon_root / "512x512" / "apps" / f"{app_id}.png"

    app_icon = Path(destdir / f"{app_id}.png")
    if app_icon.exists():
        app_icon.unlink()
    app_icon.symlink_to(hd_icon)

    dir_icon = Path(destdir / ".DirIcon")
    if dir_icon.exists():
        dir_icon.unlink()
    dir_icon.symlink_to(f"{app_id}.png")
else:
    # Fallback to SVG
    print("rsvg-convert not found. Will use the SVG variant as AppImage icon")
    print("This will render improperly on KDE and some other SVG renderers")
    hd_icon = icon_root / "scalable" / "apps" / f"{app_id}.svg"

    app_icon = Path(destdir / f"{app_id}.svg")
    if app_icon.exists():
        app_icon.unlink()
    app_icon.symlink_to(hd_icon)

    dir_icon = Path(destdir / ".DirIcon")
    if dir_icon.exists():
        dir_icon.unlink()
    dir_icon.symlink_to(f"{app_id}.svg")

# Vendor icon theme
adwaita_icons_src = gtk_root / "share" / "icons" / "Adwaita"
adwaita_icons = datadir / "icons" / "Adwaita"
shutil.copytree(adwaita_icons_src, adwaita_icons, dirs_exist_ok=True)
subprocess.run(
    ["gtk4-update-icon-cache", "-q", "-t", "-f", datadir / "icons" / "hicolor"]
)

# Vendor GtkSource data files
gtksource_src = gtk_root / "share" / "gtksourceview-5"
gtksource = datadir / "gtksourceview-5"
shutil.copytree(gtksource_src, gtksource, dirs_exist_ok=True)

# Vendor GTK schemas
glib_schemas_src = gtk_root / "share" / "glib-2.0" / "schemas"
glib_schemas = datadir / "glib-2.0" / "schemas"
for root, _, files in glib_schemas_src.walk():
    for file in files:
        if file.startswith("org.gtk."):
            src_file = root / file
            dest_file = glib_schemas / file
            shutil.copy(src_file, dest_file)
subprocess.run(["glib-compile-schemas", datadir / "glib-2.0" / "schemas"])
