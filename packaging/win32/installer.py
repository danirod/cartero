#!/usr/bin/env python3

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
