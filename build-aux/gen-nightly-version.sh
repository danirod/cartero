#!/bin/bash

cd "$(dirname "$0")/.."

VERSION=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[] | select(.name == "cartero") | .version')
DATE=$(date +%Y%m%d)

if [ -d .git ]; then
    HASH=$(git rev-parse --short HEAD)
    echo $VERSION-nightly-$DATE-$HASH
else
    echo $VERSION-nightly-$DATE
fi