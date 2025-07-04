#!/bin/bash

cd "$(dirname "$0")/.."

VERSION=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[] | select(.name == "cartero") | .version')
DATE=$(date +%Y%m%d)
echo $VERSION-nightly-$DATE