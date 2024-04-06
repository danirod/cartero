#!/bin/sh

cd "$(dirname "$0")/.."

meson_flags="--prefix=$PWD/install"
if [ -d build ]; then
        meson_flags="$meson_flags --reconfigure"
fi
meson setup build $meson_flags

ninja -C build
ninja -C build install
