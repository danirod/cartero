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

function echo_po {
    rev=$1
    locale=$2
    # This thing rocks: https://gist.github.com/SebCorbin/d196a96b1b5f30f3c3947c3d35fe420b
    # Also, the sed is used to avoid diffing removed lines. Technically they are still
    # part of the dictionary and sometimes they are added back in a future version.
    git show $rev:po/$locale.po | sed 's/^#~ //' | msgcat --no-location --no-wrap --sort-output -
}

function po_diff {
    first_rev=$1
    second_rev=$2
    locale=$3
    diff -u <(echo_po $first_rev $locale) <(echo_po $second_rev $locale)
}

function diff_version {
    old_rev=$1
    new_rev=$2

    for locale in $(linguas $new_rev); do
        if po_diff $old_rev $new_rev $locale | awk -f build-aux/translation-diffs.awk ; then
            echo $locale
        fi
    done
}

function last_version {
    old_rev=$1

    for locale in $(linguas HEAD); do
        if po_diff $old_rev HEAD $locale | awk -f build-aux/translation-diffs.awk ; then
            echo $locale
        fi
    done
}

for tag in $(git tag); do\
    [ -d "NEWS.d/${tag/v/}" ] || continue
    target=NEWS.d/${tag/v/}/translated

    if [ -z $LAST_TAG ]; then
        # It is the first commit. Use the root commit as the basis.
        root=$(git rev-list --max-parents=0 HEAD | tail -n1)
        diff_version $root $tag > $target
    else
        diff_version $LAST_TAG $tag > $target
    fi
    LAST_TAG=$tag
done

last_version $LAST_TAG > NEWS.d/unreleased/translated
