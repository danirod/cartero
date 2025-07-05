#!/usr/bin/env python3

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
        self_dir / ".." / "GPL-3.0.rtf",
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
