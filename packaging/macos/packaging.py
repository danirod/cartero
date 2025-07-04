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
    if not identity:
        args = ["codesign", "--sign", "-", "--force", "--preserve-metadata=entitlements,requirements,flags,runtime", path]
    else:
        args = ["codesign", "-v", "-f", "--timestamp", "--options=runtime", "--sign", identity, path]
    subprocess.run(args)  

# Environment variables for main paths
destdir = var_lib("DESTDIR")
source_dir = var_lib("MESON_SOURCE_ROOT")
build_dir = var_lib("MESON_BUILD_ROOT")
root_dir = var_lib("MESON_INSTALL_PREFIX")
install_dir = var_lib("MESON_INSTALL_DESTDIR_PREFIX")
template_dir = Path(__file__).parent

parser = ArgumentParser(description='Package an application bundle')
parser.add_argument('-i', '--identity', help='The digital identity for codesigning')
args = parser.parse_args()

app = install_dir / "Cartero.app"
if app.exists():
    shutil.rmtree(app)
for dmg in install_dir.glob("*.dmg"):
    dmg.unlink()

# Get the list of directories currently present in install_dir
dirs = [l.name for l in install_dir.glob("*")]

# Wrap them as Resources.
resources_dir = app / "Contents" / "Resources"
resources_dir.mkdir(parents=True, exist_ok=True)
for rdir in dirs:
    shutil.move(install_dir / rdir, resources_dir / rdir)

# For cosmetic purposes, rename bin/cartero to bin/Cartero
# (the file name is used for the "About" entry in the menu bar)
if (resources_dir / "bin" / "cartero").exists():
    shutil.move(resources_dir / "bin" / "cartero", resources_dir / "bin" / "Cartero")

# Copy the launcher
macos_dir = app / "Contents" / "MacOS"
macos_dir.mkdir(parents=True, exist_ok=True)
app_run = macos_dir / "Cartero"
shutil.copy(template_dir / "AppRun.sh", app_run)
app_run.chmod(0o755)

# Move additional resources
for icns in ['Cartero.icns', 'Cartero-request.icns']:
    shutil.copy(template_dir / icns, resources_dir / icns)

# Copy the .plist
plist_src = build_dir / "packaging" / "macos" / "Info.plist"
plist_target = app / "Contents" / "info.plist"
shutil.copy(plist_src, plist_target)

# Sign the application
for file in macos_dir.glob("*"):
    sign(args.identity, file)
for file in (resources_dir / "bin").glob("*"):
    sign(args.identity, file)
for file in (resources_dir / "lib").glob("**/*.so"):
    sign(args.identity, file)
for file in (resources_dir / "lib").glob("**/*.dylib"):
    sign(args.identity, file)
sign(args.identity, app / "Contents" / "Info.plist")
sign(args.identity, app)