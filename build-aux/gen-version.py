#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors
#
# Generates the version number for meson.build based on what the Cargo.toml
# says. If the --nightly otpion is given or the CARTERO_NIGHTLY_VERSION env
# var is set, it will treat the version number as nightly and thus add the
# compilation date too. (Meson will add the Git hash in development profile).

import datetime
import os

try:
    import tomllib
except ImportError:
    from pip._vendor import tomli as tomllib

from argparse import ArgumentParser, BooleanOptionalAction
from pathlib import Path

parser = ArgumentParser(description="Generate the version number")
parser.add_argument("--nightly", action=BooleanOptionalAction)
parser.set_defaults(nightly=False)
args = parser.parse_args()

root_dir = Path(__file__).parent.parent
cargo_file = root_dir / "Cargo.toml"
cargo_doc = tomllib.loads(cargo_file.read_text())
cargo_version = cargo_doc["workspace"]["package"]["version"]

if cargo_version.endswith(".0"):
    cargo_version = cargo_version[:-2]

if args.nightly or os.environ.get("CARTERO_NIGHTLY_VERSION"):
    now = datetime.datetime.now(datetime.UTC)
    date = now.strftime("%Y%m%d")
    cargo_version = f"{cargo_version}-nightly.{date}"

print(cargo_version)
