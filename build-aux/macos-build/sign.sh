#!/bin/bash

set -e

if [ -z "$CODESIGN_IDENTITY" ]; then
        echo "Please point CODESIGN_IDENTITY to the key ID that you get when running"
        echo "the following command, and export the environment variable:"
        echo
        echo "    security find-identity -v -p codesigning"
        exit 1
fi

if [[ "$1" == "-c" ]]; then
        check=1
        shift 1
fi

app_bundle="$1"

if ! [ -d "$app_bundle" ] || [ ! -d "$app_bundle/Contents/MacOS" ]; then
        echo "Not a valid application"
        exit 1
fi

sign() {
        if [[ $check == "1" ]]; then
                codesign -dv --verbose=4 "$1"
        else
                codesign -v -f --timestamp --options=runtime --sign "$CODESIGN_IDENTITY" "$1"
        fi
}

for bin in $app_bundle/Contents/MacOS/*; do
        if [ -x "$bin" ]; then
                sign "$bin"
        fi
done

for lib in $(find $app_bundle/Contents/Resources/lib -type f -name "*.so" -or -name "*.dylib"); do
        sign "$lib"
done

sign "$app_bundle"
