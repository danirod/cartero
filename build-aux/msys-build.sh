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

case "$1" in
        devel)
                MSYS2_ARG_CONV_EXCL="--prefix=" meson setup build --prefix="/" -Dprofile=development -Ddecorations=no-csd
                ;;
        stable)
                MSYS2_ARG_CONV_EXCL="--prefix=" meson setup build --prefix="/" -Dprofile=default -Ddecorations=no-csd
                ;;
        *)
                echo "Usage: $0 [devel / stable]"
                exit 1
                ;;
esac

ninja -C build
rm -rf $PWD/build/cartero-win32
DESTDIR=$PWD/build/cartero-win32 ninja -C build install

cd $PWD/build/cartero-win32
mkdir -p {lib,share}

cp $(ldd bin/cartero.exe | grep "$MINGW_PREFIX" | awk '{ print $3 }') bin/

cp $MINGW_PREFIX/bin/gdbus.exe bin/
cp $MINGW_PREFIX/bin/gspawn-win64-helper.exe bin/

cp -RTn $MINGW_PREFIX/lib/gdk-pixbuf-2.0 lib/gdk-pixbuf-2.0
cp -RTn $MINGW_PREFIX/share/glib-2.0 share/glib-2.0
cp -RTn $MINGW_PREFIX/share/icons/Adwaita share/icons/Adwaita
cp -RTn $MINGW_PREFIX/share/icons/hicolor share/icons/hicolor
cp -RTn $MINGW_PREFIX/share/gtksourceview-5 share/gtksourceview-5

cp $(ldd lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll | grep "$MINGW_PREFIX" | awk '{ print $3 }' | sort | uniq) bin/

glib-compile-schemas.exe share/glib-2.0/schemas
gtk4-update-icon-cache.exe -t share/icons/hicolor
