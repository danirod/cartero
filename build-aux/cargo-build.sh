#!/bin/sh
#
# This script can be used to automatically pack the resources
# and locales and place them in the default cargo-build
# compilation directory (a.k.a. target), if you don't want to
# use meson.

cd "$(dirname "$0")/.."

cargo build

meson_opts="$@"
if [ -d build ]; then
    meson_opts="--reconfigure $meson_opts"
fi

ninja -C build data/cartero.gresource
ninja -C build cartero-gmo
mkdir -p target/share/cartero
cp -R build/data/cartero.gresource target/share/cartero
cp -R build/po target/share/locale
