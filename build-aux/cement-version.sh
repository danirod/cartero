#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors

set -eu

# NOTE: this command will fail unless meson >= 1.4.0
$MESONREWRITE -V --sourcedir="$MESON_PROJECT_DIST_ROOT" kwargs set project / version "$1"
