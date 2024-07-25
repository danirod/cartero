#!/bin/bash

set -e
cd "$(dirname "$0")/.."

function package_app() {
        if ! [ -d "$1" ]; then
                echo "Directory $1 not found, skipping"
                return
        fi

        mkdir build/cartero-darwin-dmg
        cp -R "$1" build/cartero-darwin-dmg
        ln -s /Applications build/cartero-darwin-dmg/Applications
        hdiutil create -srcFolder build/cartero-darwin-dmg -volname "Cartero" -o "$2"
        rm -rf build/cartero-darwin-dmg
}

package_app "build/cartero-darwin/Cartero.app" "build/Cartero-0.1.dmg"
package_app "build/cartero-darwin/Cartero (Devel).app" "build/Cartero-0.1-devel.dmg"
