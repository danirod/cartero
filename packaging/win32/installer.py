#!/usr/bin/env python3

# A stub to call ISCC.exe with the proper environment, given that ISCC.exe
# doesn't know about the environment variables defined in our environment.

import os
import sys
import subprocess

if len(sys.argv) < 2:
    print(f"Usage: {sys.argv[0]} <installer.iss>")
    sys.exit(1)
_, iss = sys.argv[0:2]

args = ['iscc.exe', "/DSOURCE_DIR=" + os.environ.get("MESON_INSTALL_DESTDIR_PREFIX"), "/O" + os.environ.get("MESON_INSTALL_DESTDIR_PREFIX"), iss]
subprocess.run(args)