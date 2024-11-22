#!/bin/bash

set -e
cd "$(dirname "$0")/.."

case "$1" in
  devel)
    MESON_FLAGS="-Dprofile=development"
    ICON_PATH="AppDir/usr/share/icons/hicolor/scalable/apps/es.danirod.Cartero.Devel.svg"
    DESKTOP_PATH="AppDir/usr/share/applications/es.danirod.Cartero.Devel.desktop"
    ;;
  stable)
    MESON_FLAGS="-Dprofile=default"
    ICON_PATH="AppDir/usr/share/icons/hicolor/scalable/apps/es.danirod.Cartero.svg"
    DESKTOP_PATH="AppDir/usr/share/applications/es.danirod.Cartero.desktop"
    ;;
  *)
    echo "Usage: $0 [devel / stable]"
    exit 1
    ;;
esac

meson setup build --prefix="/" $MESON_FLAGS
ninja -C build
DESTDIR=$PWD/build/appimagetool/AppDir/usr ninja -C build install

cd build/appimagetool

# Vendor extra files
if [ -d /usr/share/icons/Adwaita ]; then
  mkdir -p AppDir/usr/share/icons
  cp -rv /usr/share/icons/Adwaita AppDir/usr/share/icons
  gtk4-update-icon-cache -q -t -f AppDir/usr/share/icons/Adwaita
else
  echo "Warning: cannot vendor Adwaita icons"
fi
if [ -d /usr/share/themes/Adwaita ]; then
  mkdir -p AppDir/usr/share/themes
  cp -rv /usr/share/themes/Adwaita AppDir/usr/share/themes
else
  echo "Warning: cannot vendor Adwaita themes"
fi
if [ -d /usr/share/gtksourceview-5 ]; then
  cp -rv /usr/share/gtksourceview-5 AppDir/usr/share/
else
  echo "Warning: cannot vendor GtkSourceView 5 data files"
fi
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
  
# Patch the hook in order to support Adwaita theme.
sed -i '/GTK_THEME/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh
sed -i '/GDK_BACKEND/d' AppDir/apprun-hooks/linuxdeploy-plugin-gtk.sh

# Recompile with the changes.
./appimagetool-x86_64.AppImage AppDir