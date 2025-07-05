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
import platform
import re
import sys
import shutil
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


def which(app):
    path = shutil.which(app) or panic(f"Executable {app} not found")
    return Path(path)


def ldconfig_p():
    VALID_ARCHS = {"x86_64": "x86-64", "aarch64": "aarch64"}
    output = subprocess.check_output(["ldconfig", "-p"]).decode("utf-8")
    arch = VALID_ARCHS[platform.machine()]
    lines = [line for line in output.splitlines() if arch in line]
    output = "\n".join(lines)
    groups = re.finditer(r"\t(.*)\s\(.*\) => (.*)", output)
    return dict([(m.group(1), Path(m.group(2)).resolve()) for m in groups])


# Required to copy files such as AppRun
template_path = Path(__file__).parent

# Extract environment variables
destdir = var_lib("DESTDIR")
root_dir = var_lib("MESON_INSTALL_PREFIX")
install_dir = var_lib("MESON_INSTALL_DESTDIR_PREFIX")
source_dir = var_lib("MESON_SOURCE_ROOT")

# Common directories for the installer
bindir = install_dir / "bin"
libdir = install_dir / "lib"
datadir = install_dir / "share"

# patchelf will take care of relocating executables and libraries
patchelf = which("patchelf")

# ldconfig catalog will be used to vendor shared libraries
ldconfig = ldconfig_p()

# Some information has to be infered from the argv still.
# TODO: Parse the metainfo file to avoid having to do this.
if len(sys.argv) < 2:
    print(f"Missing arguments: {sys.argv[0]} [app_id]")
    sys.exit(1)
_, app_id = sys.argv


def patchelf_needed(path):
    """Wraps a call to patchelf --print-needed"""
    print(f"Querying needed for {path}...")
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
        dep_file = ldconfig[dep_name]
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

# Recursively relocate dependency tree
while len(new_deps) > 0:
    next_deps = set()
    for new_dep in new_deps:
        new_dep_path = libdir / new_dep
        next_deps = next_deps.union(copy_shared_libraries(new_dep_path))
    new_deps = next_deps

# The following components are part of the glibc library and have to be copied
# together. You cannot just copy libc.so.6 or ld-linux-$(arch).so.2 and expect
# things to work. Everything has to go in a single pack.
ld_so = Path(shutil.which("ld.so")).resolve().name
libc_components = [
    ld_so,
    "libanl.so.1",
    "libBrokenLocale.so.1",
    "libc_malloc_debug.so.0",
    "libc.so.6",
    "libdl.so.2",
    "libm.so.6",
    "libmvec.so.1",
    "libnsl.so.1",
    "libnss_compat.so.2",
    "libnss_db.so.2",
    "libnss_dns.so.2",
    "libnss_files.so.2",
    "libnss_hesiod.so.2",
    "libpthread.so.0",
    "libresolv.so.2",
    "librt.so.1",
    "libthread_db.so.1",
    "libutil.so.1",
]
for libc_component in libc_components:
    if libc_component in ldconfig:
        source_lib = ldconfig[libc_component]
        target_path = libdir / libc_component
        shutil.copy(source_lib, target_path)

# Bring the gdk-pixbuf loaders
loaders_dir = Path(pkg_config("gdk-pixbuf-2.0", "gdk_pixbuf_moduledir"))
pixbuf_moduledir = libdir / "gdk-pixbuf-2.0" / "2.10.0"
if not pixbuf_moduledir.exists():
    pixbuf_moduledir.mkdir(parents=True, exist_ok=True)
pixbuf_loaders = pixbuf_moduledir / "loaders"
shutil.copytree(loaders_dir, pixbuf_loaders)

# Generate a new loaders.cache file
query_loaders = pkg_config("gdk-pixbuf-2.0", "gdk_pixbuf_query_loaders")
loaders_cache = subprocess.check_output(
    [query_loaders],
    env={
        "GDK_PIXBUF_MODULEDIR": pixbuf_loaders,
    },
).decode("utf-8")
loaders_cache = loaders_cache.replace(str(pixbuf_loaders) + "/", "")
with open(pixbuf_moduledir / "loaders.cache", mode="w") as file:
    file.write(loaders_cache)

# With the loaders.cache file written, reposition the loaders.
# Relocate the loaders
for loader in pixbuf_loaders.glob("*.so"):
    shutil.move(loader, libdir / loader.name)
    patchelf_set_rpath(libdir / loader.name, "$ORIGIN")
    deps = set([loader.name])
    while len(deps) > 0:
        next_deps = set()
        for dep in deps:
            dep_path = libdir / dep
            new_deps = copy_shared_libraries(dep_path)
            next_deps = next_deps.union(new_deps)
        deps = next_deps

# Copy AppRun script
apprun_in = template_path / "AppRun.in"
apprun = apprun_in.read_text().replace("%LOADER%", ld_so)
apprun_file = destdir / "AppRun"
apprun_file.write_text(apprun)
apprun_file.chmod(0o755)

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
gtk_root = Path(pkg_config("gtk4", "prefix"))
adwaita_icons_src = gtk_root / "share" / "icons" / "Adwaita"
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
subprocess.run(["glib-compile-schemas", datadir / "glib-2.0" / "schemas"])

# Vendor additional locales
linguas_file = source_dir / "po" / "LINGUAS"
linguas = [
    lingua.strip()
    for lingua in open(linguas_file).readlines()
    if not lingua.startswith("#")
]
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
        mo_file = glib_root / "share" / "locale" / lang / "LC_MESSAGES" / f"{pkg}.mo"
        if mo_file.exists():
            print(f"Copying .mo file for {pkg} ({lang})...")
            shutil.copy(
                mo_file, datadir / "locale" / lang / "LC_MESSAGES" / f"{pkg}.mo"
            )
        else:
            print(f".mo file for {pkg} not found for locale {lang}, skipping...")
