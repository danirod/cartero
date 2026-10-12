#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors

# This script attempts to normalize a translation file in a way that is not
# meaningful for anything, but can be used safely as part of a diff to check if
# a gettext translation has changed. Prints to standard output. Capture this
# somewhere!
#
# This is only used in CI and for build-aux/check-local.sh to see if the PO
# files are outdated. Do not rely on this for anything else!
#
# (You might be wondering why not just diffing the .po files. The explanation
# is that it is not easy to achieve this in a way that breaks subtly. Running
# cartero-pot or cartero-update-po in meson will always update the header entry
# with the current date, so the file will always have Git changes. I used to
# test for diff lines that start with `msgid`, but this is not safe when the
# msgid string is so long that it is split in multiple lines.)

set -euo pipefail
cd "$(dirname "$0")/.."

# To use a common baseline when working with languages. Do not remove!
export LC_ALL=C.UTF-8

function print_file() {
        echo "== file $1 starts here == "
        msgattrib --no-obsolete "$1" | msgexec bash -c ' \
                if [[ -n "$MSGEXEC_MSGID" ]] ; then
                        printf "%q %q %q %q %q\n" \
                                "${MSGEXEC_MSGCTXT+x}" \
                                "${MSGEXEC_MSGCTXT-}" \
                                "$MSGEXEC_MSGID" \
                                "${MSGEXEC_MSGID_PLURAL+x}" \
                                "${MSGEXEC_MSGID_PLURAL-}"
                        fi' | sort -u
        echo "== file $1 ends here == "
}

print_file po/cartero.pot

for f in po/*.po; do
        print_file $f
done
