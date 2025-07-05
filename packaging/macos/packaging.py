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


def sign(identity: str, path: Path):
    if not identity or identity == "-":
        args = [
            "codesign",
            "--sign",
            "-",
            "--force",
            "--preserve-metadata=entitlements,requirements,flags,runtime",
            path,
        ]
    else:
        args = [
            "codesign",
            "-v",
            "-f",
            "--timestamp",
            "--options=runtime",
            "--sign",
            identity,
            path,
        ]
    subprocess.run(args)


def shared_libraries(path):
    args = ["otool", "-L", path]
    output = subprocess.check_output(args).decode("utf-8")
    otool_lib = r"\t(.*) \(.*\)"
    return re.findall(otool_lib, output)


def relink_dependency(path, old, new):
    """Wraps a call to install_name_tool"""
    args = ["install_name_tool", "-change", old, new, path]
    print("relinking:", args)
    subprocess.run(args)


# Environment variables for main paths
destdir = var_lib("DESTDIR")
source_dir = var_lib("MESON_SOURCE_ROOT")
build_dir = var_lib("MESON_BUILD_ROOT")
root_dir = var_lib("MESON_INSTALL_PREFIX")
install_dir = var_lib("MESON_INSTALL_DESTDIR_PREFIX")
template_dir = Path(__file__).parent

parser = ArgumentParser(description="Package an application bundle")
parser.add_argument("-i", "--identity", help="The digital identity for codesigning")
args = parser.parse_args()

app = install_dir / "Cartero.app"
if app.exists():
    shutil.rmtree(app)

# Directories of interest within the app bundle
app_contents = app / "Contents"
app_macos = app_contents / "MacOS"
app_resources = app_contents / "Resources"

# bin/cartero => .app/Contents/MacOS/Cartero
app_macos.mkdir(exist_ok=True, parents=True)
bin_cartero = app_macos / "Cartero"
shutil.move(install_dir / "bin" / "cartero", bin_cartero)
(install_dir / "bin").rmdir()

# bin/cartero has moved locations, so we have to update the path to the shared libraries.
# TODO: Can't just switch to rpath to avoid having to do this?
loader_libs = [
    dep for dep in shared_libraries(bin_cartero) if dep.startswith("@loader_path")
]
for lib in loader_libs:
    new_lib = lib.replace("@loader_path/../lib", "@loader_path/../Resources/lib")
    relink_dependency(bin_cartero, lib, new_lib)

# lib,opt,share => .app/Contents/Resources
app_resources.mkdir(exist_ok=True, parents=True)
for res_dir in ["lib", "opt", "share"]:
    if (install_dir / res_dir).exists():
        shutil.move(install_dir / res_dir, app_resources / res_dir)

# Move additional resources
for icns in ["Cartero.icns", "Cartero-request.icns"]:
    shutil.copy(template_dir / icns, app_resources / icns)

# Copy the .plist
plist_src = build_dir / "packaging" / "macos" / "Info.plist"
plist_target = app_contents / "Info.plist"
shutil.copy(plist_src, plist_target)

# Sign the application
for file in app_macos.glob("*"):
    sign(args.identity, file)
for file in (app_resources / "lib").glob("**/*.so"):
    sign(args.identity, file)
for file in (app_resources / "lib").glob("**/*.dylib"):
    sign(args.identity, file)
sign(args.identity, app_contents / "Info.plist")
sign(args.identity, app)
