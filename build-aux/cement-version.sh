#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors

set -eu

# NOTE: this command will fail unless meson >= 1.4.0
$MESONREWRITE -V --sourcedir="$MESON_PROJECT_DIST_ROOT" kwargs set project / version "$2"

# There is static data in the snapcraft.yaml package too, so it is time to rewrite.
sed -i -e "s/version: .*/version: '$2'/" \
    -i -e "s/es.danirod.Cartero.Devel/$1/g" \
    "$MESON_PROJECT_DIST_ROOT/snapcraft.yaml"
