#!/bin/bash

set -e

cd "$(dirname "$0")"

if ! [ -d jhb ]; then
    git clone https://gitlab.com/dehesselle/jhb
fi

# NOTE: Remove this once jhb receives meson >= 1.8.4
export VERSION=1.3+meson184
sed -i '' 's|https://gitlab.com/api/v4/projects/35965804/packages/generic/jhb/|https://github.com/danirod/runner-jhb/releases/download/|' jhb/etc/jhb.conf.d/release.sh

# Force compatibility with macOS 11.
MACOSX_DEPLOYMENT_TARGET=11.0

source jhb/etc/jhb.conf.sh

# Compile SDK
rm -rf $VER_DIR
jhb/usr/bin/bootstrap
jhb/usr/bin/jhb configure modulesets/gnome-sdk.modules
jhb/usr/bin/jhb build gnome-sdk

function relocate {
	file="$1"
	for rpath in $(otool -L "$file" | grep '@rpath/' | awk '{ print $1 }'); do
		new_path=${rpath/@rpath/$VER_DIR/lib}
		install_name_tool -change "$rpath" "$new_path" "$file"
	done
}

# Relocate some dependencies
for file in $(cat $VER_DIR/var/jhbuild/manifests/* | sort -h); do
	ftype=$(file "$VER_DIR/$file")
	if [[ $ftype == *"Mach-O 64-bit executable"* ]]; then
		echo Relocating $VER_DIR/$file as exe
		relocate "$VER_DIR/$file"
	elif [[ $ftype == *"Mach-O 64-bit dynamically linked shared library"* ]]; then
		echo Relocating $VER_DIR/$file as lib
		install_name_tool -id "$(greadlink -f "$VER_DIR/$file")" "$VER_DIR/$file"
		relocate "$VER_DIR/$file"
	elif [[ $ftype == *"Mach-O 64-bit bundle"* ]]; then
		echo Relocating $VER_DIR/$file as bundle
		relocate "$VER_DIR/$file"
	fi
done

# GTK post install
echo "Recompiling system schemas..."
$VER_DIR/bin/glib-compile-schemas $VER_DIR/share/glib-2.0/schemas/

echo "Reloading icon caches..."
for theme in $VER_DIR/share/icons/*; do
    $VER_DIR/bin/gtk4-update-icon-cache $theme
done

echo "Reloading pixbuf caches..."
GDK_PIXBUF_MODULEDIR=$VER_DIR/lib/gdk-pixbuf-2.0/2.10.0/loaders/ $VER_DIR/bin/gdk-pixbuf-query-loaders > $VER_DIR/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache

# Finalize system
jhb/usr/bin/archive remove_nonessentials
