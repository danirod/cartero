#!/bin/bash

set -e
cd "$(dirname "$0")/.."
export GETTEXT_DIR=$(brew --prefix)/opt/gettext

brew install meson gtk4 gtksourceview5 desktop-file-utils pygobject3 libadwaita adwaita-icon-theme gettext

case "$1" in
  devel)
    BUNDLE_ID="es.danirod.Cartero.Devel"
    APP_NAME="Cartero (Devel)"
    APP_VERSION="0.1.0"
    MESON_FLAGS="-Dprofile=development"
    ICON_PATH="$PWD/data/icons/scalable/apps/es.danirod.Cartero.Devel.svg"
    ;;
  stable)
    BUNDLE_ID="es.danirod.Cartero"
    APP_NAME="Cartero"
    APP_VERSION="0.1.0"
    MESON_FLAGS="-Dprofile=default"
    ICON_PATH="$PWD/data/icons/scalable/apps/es.danirod.Cartero.svg"
    ;;
  *)
    echo "Usage: $0 [devel / stable]"
    exit 1
    ;;
esac

meson setup build --prefix="/" -Ddecorations=no-csd $MESON_FLAGS
ninja -C build

APP_ROOT="$PWD/build/cartero-darwin/$APP_NAME.app"
RESOURCES_ROOT="$APP_ROOT/Contents/Resources"

rm -rf "$APP_ROOT"

# Prepare share directory
mkdir -p "$RESOURCES_ROOT/share/glib-2.0/schemas"
cp -R $(brew --prefix)/opt/glib/share/glib-2.0/schemas/* "$RESOURCES_ROOT/share/glib-2.0/schemas"
cp -R $(brew --prefix)/opt/gtk4/share/glib-2.0/schemas/* "$RESOURCES_ROOT/share/glib-2.0/schemas"
cp -R $(brew --prefix)/opt/gtksourceview5/share/gtksourceview-5 "$RESOURCES_ROOT/share"
mkdir -p "$RESOURCES_ROOT/share/icons"
cp -R $(brew --prefix)/opt/adwaita-icon-theme/share/icons/Adwaita "$RESOURCES_ROOT/share/icons"
cp -R $(brew --prefix)/opt/hicolor-icon-theme/share/icons/hicolor "$RESOURCES_ROOT/share/icons"

# Compile share directory
DESTDIR="$RESOURCES_ROOT" ninja -C build install
glib-compile-schemas "$RESOURCES_ROOT/share/glib-2.0/schemas"
gtk4-update-icon-cache -q -t -f "$RESOURCES_ROOT/share/icons/hicolor"
gtk4-update-icon-cache -q -t -f "$RESOURCES_ROOT/share/icons/Adwaita"

# Mangle bin directory
mv "$RESOURCES_ROOT/bin" "$APP_ROOT/Contents/MacOS"

# This part is a mess. To anyone trying to understand what does this do, here is an overview.
#
# It has to find every shared library used by the executable that comes from Homebrew, and vendor
# it into the Resources/lib directory, so that it is packed with the application. Otherwise,
# Cartero won't run because the shared library has to be somewhere.
function get_absolute_nonrelocated() {
  otool -L "$1" | grep $(brew --prefix) | awk '{ print $1 }'
}
function get_relative_nonrelocated() {
  otool -L "$1" | grep "@loader_path/../../../.." | awk '{ print $1 }'
}

mkdir -p "$RESOURCES_ROOT/lib"

# Relocate Cartero executable
for dep in $(get_absolute_nonrelocated "$APP_ROOT/Contents/MacOS/cartero"); do
  dep_name=$(basename "$dep")
  cp -v $dep "$RESOURCES_ROOT/lib"
  install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_name" "$APP_ROOT/Contents/MacOS/cartero"
done

# Copy and relocate gdk-pixbuf-2.0 loaders for gdk-pixbuf-2.0 and librsvg
mkdir -p "$RESOURCES_ROOT/lib/gdk-pixbuf-2.0/2.10.0/loaders"
cp $(brew --prefix)/opt/gdk-pixbuf/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so "$RESOURCES_ROOT/lib/gdk-pixbuf-2.0/2.10.0/loaders"
cp $(brew --prefix)/opt/librsvg/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so "$RESOURCES_ROOT/lib/gdk-pixbuf-2.0/2.10.0/loaders"
find "$RESOURCES_ROOT/lib/gdk-pixbuf-2.0/2.10.0/loaders" -name '*.so' | while read f; do
  file_name=$(basename "$f")
  echo "FILE: $file_name"
  install_name_tool -id "@executable_path/../Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders/$file_name" "$f"
  for dep in $(get_absolute_nonrelocated "$f"); do
    dep_name=$(basename "$dep")
    if ! [ -f "$RESOURCES_ROOT/lib/$dep_name" ]; then
      cp -v "$dep" "$RESOURCES_ROOT/lib"
    fi
    install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_name" "$f"
  done
  for dep in $(get_relative_nonrelocated "$f"); do
    real_name=$(get_relative_nonrelocated "$f" | sed "s|@loader_path/../../../..|$(brew --prefix)|")
    dep_name=$(basename "$dep")
    if ! [ -f "$RESOURCES_ROOT/lib/$dep_name" ]; then
      cp -v "$real_name" "$RESOURCES_ROOT/lib"
    fi
    install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_name" "$f"
  done
done

# Copy then relocate additional dylibs
function concat_absolute_dylibs() {
  find "$RESOURCES_ROOT/lib" -name '*.dylib' | while read f; do
    otool -L "$f"
  done | grep $(brew --prefix) | awk '{ print $1 }' | sort | uniq
}
function concat_relative_dylibs() {
  find "$RESOURCES_ROOT/lib" -name '*.dylib' | while read f; do
    otool -L "$f"
  done | grep "@loader_path/../../../.." | awk '{ print $1 }' | sort | uniq
}
function any_pending_dylibs() {
  [[ -n "$(concat_absolute_dylibs)" ]] && return 0
  [[ -n "$(concat_relative_dylibs)" ]] && return 0
  return 1
}

while any_pending_dylibs; do
  [[ -n "$(concat_absolute_dylibs)" ]] && cp -vf $(concat_absolute_dylibs) "$RESOURCES_ROOT/lib"
  [[ -n "$(concat_relative_dylibs)" ]] && cp -vf $(concat_relative_dylibs | sed "s|@loader_path/../../../..|$(brew --prefix)|") "$RESOURCES_ROOT/lib"

  find "$RESOURCES_ROOT/lib" -name '*.dylib' | while read f; do
    file_name=$(basename "$f")
    install_name_tool -id "@executable_path/../Resources/lib/$file_name" "$f"
    for dep in $(get_absolute_nonrelocated "$f"); do
      dep_name=$(basename "$dep")
      if ! [ -f "$RESOURCES_ROOT/lib/$dep_name" ]; then
        cp "$dep" "$RESOURCES_ROOT/lib"
      fi
      # install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_name" "$f"
      install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_name" "$f"
    done
    for dep in $(get_relative_nonrelocated "$f"); do
      real_name=$(get_relative_nonrelocated "$f" | sed "s|@loader_path/../../../..|$(brew --prefix)|")
      dep_name=$(basename "$dep")
      if ! [ -f "$RESOURCES_ROOT/lib/$dep_name" ]; then
        cp "$real_name" "$RESOURCES_ROOT/lib"
      fi
      # install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_name" "$f"
      install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_name" "$f"
    done
  done
done

# Create cache
GDK_PIXBUF_MODULEDIR="$(brew --prefix)/opt/gdk-pixbuf/lib/gdk-pixbuf-2.0/2.10.0/loaders" gdk-pixbuf-query-loaders | sed "s|\".*/lib/gdk-pixbuf-2.0|\"@executable_path/../Resources/lib/gdk-pixbuf-2.0|" > "$RESOURCES_ROOT/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"
GDK_PIXBUF_MODULEDIR="$(brew --prefix)/opt/librsvg/lib/gdk-pixbuf-2.0/2.10.0/loaders" gdk-pixbuf-query-loaders | sed "s|\".*/lib/gdk-pixbuf-2.0|\"@executable_path/../Resources/lib/gdk-pixbuf-2.0|" >> "$RESOURCES_ROOT/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"

codesign --sign - --force --preserve-metadata=entitlements,requirements,flags,runtime "$APP_ROOT/Contents/MacOS/cartero"
find "$RESOURCES_ROOT/lib" -name '*.dylib' | while read f; do
  codesign --sign - --force --preserve-metadata=entitlements,requirements,flags,runtime "$f"
done
find "$RESOURCES_ROOT/lib/gdk-pixbuf-2.0/2.10.0/loaders" -name '*.so' | while read f; do
  codesign --sign - --force --preserve-metadata=entitlements,requirements,flags,runtime "$f"
done

# These directories must exist for macOS to pick the locales (they can be empty)
mkdir -p "$RESOURCES_ROOT/en.lproj"
mkdir -p "$RESOURCES_ROOT/es.lproj"
mkdir -p "$RESOURCES_ROOT/eo.lproj"

# Create Info.plist
cat > "$APP_ROOT/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>cartero</string>
    <key>CFBundleIconFile</key>
    <string>Cartero.icns</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 The Cartero Authors</string>
    <key>CFBundleSignature</key>
    <string>Cartero</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.14</string>
  </dict>
</plist>
EOF

# Create icon
mkdir -p "$RESOURCES_ROOT/Cartero.iconset"
convert -background none -resize '!16x16' "$ICON_PATH" "$RESOURCES_ROOT/Cartero.iconset/icon_16x16.png"
convert -background none -resize '!32x32' "$ICON_PATH" "$RESOURCES_ROOT/Cartero.iconset/icon_32x32.png"
convert -background none -resize '!64x64' "$ICON_PATH" "$RESOURCES_ROOT/Cartero.iconset/icon_64x64.png"
convert -background none -resize '!128x128' "$ICON_PATH" "$RESOURCES_ROOT/Cartero.iconset/icon_128x128.png"
convert -background none -resize '!256x256' "$ICON_PATH" "$RESOURCES_ROOT/Cartero.iconset/icon_256x256.png"
convert -background none -resize '!512x512' "$ICON_PATH" "$RESOURCES_ROOT/Cartero.iconset/icon_512x512.png"
cp "$RESOURCES_ROOT/Cartero.iconset/icon_32x32.png" "$RESOURCES_ROOT/Cartero.iconset/icon_16x16@2x.png"
cp "$RESOURCES_ROOT/Cartero.iconset/icon_64x64.png" "$RESOURCES_ROOT/Cartero.iconset/icon_32x32@2x.png"
cp "$RESOURCES_ROOT/Cartero.iconset/icon_128x128.png" "$RESOURCES_ROOT/Cartero.iconset/icon_64x64@2x.png"
cp "$RESOURCES_ROOT/Cartero.iconset/icon_256x256.png" "$RESOURCES_ROOT/Cartero.iconset/icon_128x128@2x.png"
cp "$RESOURCES_ROOT/Cartero.iconset/icon_512x512.png" "$RESOURCES_ROOT/Cartero.iconset/icon_256x256@2x.png"
convert -background none -resize '!1024x1024' "$ICON_PATH" "$RESOURCES_ROOT/Cartero.iconset/icon_512x512@2x.png"
iconutil -c icns "$RESOURCES_ROOT/Cartero.iconset"
rm -rf "$RESOURCES_ROOT/Cartero.iconset"