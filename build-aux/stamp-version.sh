#!/bin/bash

cd "$(dirname "$0")/.."

if [[ -z "$1" ]]; then
        echo "Please provide the build directory where you have made meson setup" >&2
        echo "Usage: $0 [dir]"
        exit 1
fi

version=$(meson introspect "$1" -a | jq -r '.projectinfo.version' | sed "s/git/nightly-$(date +"%Y%m%d")/")
meson rewrite kwargs set project / version "$version"
sed -i "s/^version = .*/version = \"$version\"/" Cargo.toml
