#!/bin/bash

set -e
cd "$(dirname "$0")/.."

function package_app() {
        if ! [ -d "$1" ]; then
                echo "Directory $1 not found, skipping"
                return
        fi

        app_name="$(basename "$1")"

        mkdir -p build/cartero-darwin-dmg
        cp -Rf "$1" build/cartero-darwin-dmg
        codesign --sign - --force --deep "build/cartero-darwin-dmg/$app_name"
        ln -s /Applications build/cartero-darwin-dmg/Applications
        hdiutil create -srcFolder build/cartero-darwin-dmg -volname "Cartero" -o "$2"
        rm -rf build/cartero-darwin-dmg
}

package_app "build/cartero-darwin/Cartero.app" "build/Cartero-0.1.2.dmg"
package_app "build/cartero-darwin/Cartero (Devel).app" "build/Cartero-0.1.2-devel.dmg"
