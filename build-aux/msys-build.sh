#!/bin/bash

set -e

pacman -Sy --noconfirm --needed \
    ${MINGW_PACKAGE_PREFIX}-desktop-file-utils \
    ${MINGW_PACKAGE_PREFIX}-gcc \
    ${MINGW_PACKAGE_PREFIX}-gettext \
    ${MINGW_PACKAGE_PREFIX}-gtk4 \
    ${MINGW_PACKAGE_PREFIX}-gtksourceview5 \
    ${MINGW_PACKAGE_PREFIX}-libxml2 \
    ${MINGW_PACKAGE_PREFIX}-librsvg \
    ${MINGW_PACKAGE_PREFIX}-meson \
    ${MINGW_PACKAGE_PREFIX}-pkgconf \
    ${MINGW_PACKAGE_PREFIX}-libadwaita \
    ${MINGW_PACKAGE_PREFIX}-blueprint-compiler \
    meson

MSYS2_ARG_CONV_EXCL="--prefix=" meson setup build --prefix="${MINGW_PREFIX}"
ninja -C build
DESTDIR=$PWD/install ninja -C build install

cd $PWD/install/$MINGW_PREFIX
mkdir -p {lib,share}

cp $(ldd bin/cartero.exe | grep "$MINGW_PREFIX" | awk '{ print $3 }') bin/

cp $MINGW_PREFIX/bin/gdbus.exe bin/

cp -RTn $MINGW_PREFIX/lib/gdk-pixbuf-2.0 lib/gdk-pixbuf-2.0
cp -RTn $MINGW_PREFIX/share/glib-2.0 share/glib-2.0
cp -RTn $MINGW_PREFIX/share/icons/Adwaita share/icons/Adwaita
cp -RTn $MINGW_PREFIX/share/icons/hicolor share/icons/hicolor
cp -RTn $MINGW_PREFIX/share/gtksourceview-5 share/gtksourceview-5

cp $(ldd lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll | grep "$MINGW_PREFIX" | awk '{ print $3 }' | sort | uniq) bin/

glib-compile-schemas.exe share/glib-2.0/schemas
gtk4-update-icon-cache.exe -t share/icons/hicolor

cd ..
mv .${MINGW_PREFIX} cartero
zip -r cartero.zip cartero