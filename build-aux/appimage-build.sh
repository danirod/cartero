#!/bin/bash

# I don't know what I am doing, but here are some notes.
#
# The way this script works is by using Linuxdeploy and the GTK plugin, and then manually
# copying and relocating a shitton of libraries that Linuxdeploy-GTK doesn't pack but that
# are actually required as transitive dependencies. Additionally, I patch the Linuxdeploy
# GTK hook, make some cleanup... at this point, what's even the point on using Linuxdeploy
# if I already know what do I have to do based on what I do for macOS, for example?
#
# TODO: Remove Linuxdeploy, manually copy and relocate with patchelf every required
# dependency just like I do to relocate the macOS version of Cartero.

set -e
cd "$(dirname "$0")/.."

# Vendor base is the directory where the datafiles to copy are expected to be, such
# as the GtkSourceView themes or the Adwaita icons. This is set to /usr in case you
# are building for yourself. Set to a different value if running in a special
# environment (icon theme is installed to /usr/local or you are running Linuxbrew).
VENDOR_BASE=${VENDOR_BASE:=/usr}

case "$1" in
  devel)
    MESON_FLAGS="-Dprofile=development"
    APP_NAME="es.danirod.Cartero.Devel"
    ICON_PATH="AppDir/usr/share/icons/hicolor/scalable/apps/es.danirod.Cartero.Devel.svg"
    DESKTOP_PATH="AppDir/usr/share/applications/es.danirod.Cartero.Devel.desktop"
    ;;
  stable)
    MESON_FLAGS="-Dprofile=default"
    APP_NAME="es.danirod.Cartero"
    ICON_PATH="AppDir/usr/share/icons/hicolor/scalable/apps/es.danirod.Cartero.svg"
    DESKTOP_PATH="AppDir/usr/share/applications/es.danirod.Cartero.desktop"
    ;;
  *)
    echo "Usage: $0 [devel / stable]"
    exit 1
    ;;
esac

# This script is currently only tested for GitHub Actions.
if [ -z "$GITHUB_ACTIONS" ]; then
  echo "WARNING! This script has only been tested to run in GitHub Actions and"
  echo "it is used to build the stable and nightly versions in a well known"
  echo "environment. Running this script to create an AppImage on your computer"
  echo "is currently NOT supported and not guaranteed to work."
  echo
  echo "You should read the contents of the shell script at least once before"
  echo "running it to know what it does. I cannot help with this script because"
  echo "I don't have a clue about the inners of AppImage and its build tools."
  echo
  echo "Do you have that knowledge and know how to properly pack GTK4 + Libadwaita"
  echo "apps in a way that actually works with older versions of libc6 without"
  echo "having to use weird tricks? Please help me! Your inputs are appreciated"
  echo "Send patches or comments to https://github.com/danirod/cartero"
  echo
  echo "Last chance. Press Ctrl-C to quit the script, or Enter to start."
  read -r
fi

meson setup build --prefix="/" $MESON_FLAGS
ninja -C build
DESTDIR=$PWD/build/appimagetool/AppDir/usr ninja -C build install

cd build/appimagetool

# Apparently AppImage calls this metainfo rather than appinfo
cp -r AppDir/usr/share/appdata AppDir/usr/share/metainfo

# Prepare icon
LARGE_ICON_PATH="$ICON_PATH"
if command -v rsvg-convert 2>&1 >/dev/null; then
  # rsvg-convert exists. Let's convert the SVG icon to PNG, because apparently
  # the SVG implementation in KDE is more limited than rsvg and the icon does
  # not display properly.
  for size in 16 24 32 48 64 96 128 256 512; do
    mkdir -p AppDir/usr/share/icons/hicolor/${size}x${size}/apps/
    rsvg-convert -w $size -h $size -f png -o AppDir/usr/share/icons/hicolor/${size}x${size}/apps/$APP_NAME.png $ICON_PATH
  done
  LARGE_ICON_PATH="AppDir/usr/share/icons/hicolor/512x512/apps/$APP_NAME.png"
fi

# Vendor extra files
if [ -d $VENDOR_BASE/share/icons/Adwaita ]; then
  mkdir -p AppDir/usr/share/icons
  echo "$VENDOR_BASE/share/icons/Adwaita -> AppDir/usr/share/icons"
  cp -rL $VENDOR_BASE/share/icons/Adwaita AppDir/usr/share/icons
  gtk4-update-icon-cache -q -t -f AppDir/usr/share/icons/Adwaita
else
  echo "Warning: cannot vendor Adwaita icons"
fi
if [ -d $VENDOR_BASE/share/themes/Adwaita ]; then
  mkdir -p AppDir/usr/share/themes
  echo "$VENDOR_BASE/share/themes/Adwaita -> AppDir/usr/share/themes"
  cp -rL $VENDOR_BASE/share/themes/Adwaita AppDir/usr/share/themes
else
  echo "Warning: cannot vendor Adwaita themes"
fi
if [ -d $VENDOR_BASE/share/gtksourceview-5 ]; then
  echo "$VENDOR_BASE/share/gtksourceview-5 -> AppDir/usr/share/"
  cp -rL $VENDOR_BASE/share/gtksourceview-5 AppDir/usr/share/
else
  echo "Warning: cannot vendor GtkSourceView 5 data files"
fi
if [ -d $VENDOR_BASE/share/glib-2.0/schemas ]; then
  echo "$VENDOR_BASE/share/glib-2.0/schemas -> AppDir/usr/share/"
  cp -rL $VENDOR_BASE/share/glib-2.0/schemas/org.gtk.* AppDir/usr/share/glib-2.0/schemas
else
  echo "Warning: cannot deploy extra GTK schemas"
fi
glib-compile-schemas AppDir/usr/share/glib-2.0/schemas
gtk4-update-icon-cache -q -t -f AppDir/usr/share/icons/hicolor

# Start packaging process
[ -x appimagetool-x86_64.AppImage ] || curl -OL https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
[ -x linuxdeploy-x86_64.AppImage ] || curl -OL https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
[ -x linuxdeploy-plugin-gtk.sh ] || curl -OL https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
chmod +x appimagetool-x86_64.AppImage linuxdeploy-x86_64.AppImage linuxdeploy-plugin-gtk.sh

# First iteration
export DEPLOY_GTK_VERSION=4
./linuxdeploy-x86_64.AppImage --appdir AppDir --plugin gtk --output appimage \
  --executable AppDir/usr/bin/cartero \
  --icon-file "$ICON_PATH" \
  --desktop-file "$DESKTOP_PATH"

# Patch the hook file
# TODO: Why do I even try? Just make my own!
sed -i '/GTK_THEME/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i '/GDK_BACKEND/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i '/GSETTINGS_SCHEMA_DIR/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i '/GI_TYPELIB_PATH/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i '/GTK_EXE_PREFIX/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i '/GTK_PATH/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i '/GDK_PIXBUF_MODULE_FILE/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export GSETTINGS_SCHEMA_DIR="$APPDIR/usr/share/glib-2.0/schemas"' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export GI_TYPELIB_PATH="$APPDIR/usr/lib/girepository-1.0"' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export GTK_EXE_PREFIX="$APPDIR/usr"' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export GTK_PATH="$APPDIR/usr/lib/gtk-4.0"' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export GDK_PIXBUF_MODULE_FILE="$APPDIR/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export FONTCONFIG_PATH=/etc/fonts' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export XKB_CONFIG_ROOT=/usr/share/X11/xkb' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
echo 'export QT_XKB_CONFIG_ROOT=/usr/share/X11/xkb' >> AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh

# Bypass AppRun.wrapped
rm AppDir/AppRun.wrapped
sed -i '/exec/d' AppDir/AppRun
echo 'exec "$this_dir"/usr/bin/cartero "$@"' >> AppDir/AppRun

# Check for symlinks in /lib (specifically when built in CI)
for f in $(find AppDir/usr/lib -type l); do
  cp --remove-destination $(readlink -e "$f") "$f"
done

# Prepare to use patchelf
echo "Unpacking patchelf..."
./linuxdeploy-x86_64.AppImage --appimage-extract
mv squashfs-root linuxdeploy-root
PATCHELF=linuxdeploy-root/usr/bin/patchelf

# gdk-pixbuf-2.0
echo "Preparing GDK pixbuf cache..."
if [ -d $VENDOR_BASE/lib/gdk-pixbuf-2.0 ]; then
  for f in $VENDOR_BASE/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so; do
    loader=$(basename "$f")
    chmod u+w AppDir/usr/lib/$loader
    $PATCHELF --set-rpath '$ORIGIN' AppDir/usr/lib/$loader
  done
  mkdir -p AppDir/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders
  GDK_PIXBUF_MODULEDIR="$VENDOR_BASE/lib/gdk-pixbuf-2.0/2.10.0/loaders" gdk-pixbuf-query-loaders | sed "s|\".*/lib/gdk-pixbuf-2.0/2.10.0/loaders/|\"|" > AppDir/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache
fi

# Provide additional libraries
echo "Linking extra libraries..."
REQUIRES_MORE_DEPS=1
while [ $REQUIRES_MORE_DEPS -eq 1 ]; do
	# Unset the variable in case this is the last iteration
	REQUIRES_MORE_DEPS=0

for lib in AppDir/usr/lib/*.so AppDir/usr/lib/*.so.*; do
  # Process this dependency.
  for dep in $($PATCHELF --print-needed $lib); do
    if ! [ -f AppDir/usr/lib/$dep ] && [ -f $VENDOR_BASE/lib/$dep ]; then
      echo "$dep is missing from the distribution ($lib)"
      cp $VENDOR_BASE/lib/$dep AppDir/usr/lib
      chmod u+w AppDir/usr/lib/$dep
      $PATCHELF --set-rpath '$ORIGIN' AppDir/usr/lib/$dep
      $PATCHELF --replace-needed $dep $dep $lib
      REQUIRES_MORE_DEPS=1
    fi
  done
  done
done

# Remove linuxbrew stuff if present
[ -d AppDir/home ] && rm -rf AppDir/home
[ -d AppDir/usr/home ] && rm -rf AppDir/usr/home

# Recompile with the changes.
./appimagetool-x86_64.AppImage AppDir
