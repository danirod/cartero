#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors
#
# Annotates translations into strings in the translated file for each news.

set -x

cd "$(dirname "$0")/.."

LAST_TAG=""

function linguas {
    version=$1
    git show $version:po/LINGUAS | grep -v '^#' | grep -v en
}

function initial_version {
    version=$1
    for locale in $(linguas $version); do
        if git show $version:po/$locale.po | grep '^msgstr ' | grep -qv '^msgstr ""' ; then
            echo $locale
        fi
    done
}

function diff_version {
    version=$1
    diffspec=$2

    for locale in $(linguas $version); do
        if git diff --unified=0 $diffspec po/$locale.po | grep '^+msgstr ' | grep -qv '^+msgstr ""' ; then
            echo $locale
        fi
    done
}

for tag in $(git tag); do\
    [ -d "NEWS.d/${tag/v/}" ] || continue
    target=NEWS.d/${tag/v/}/translated

    if [ -z $LAST_TAG ]; then
        initial_version $tag > $target
    else
        diff_version $tag "$LAST_TAG...$tag" > $target
    fi
    LAST_TAG=$tag
done
