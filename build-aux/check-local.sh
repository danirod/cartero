#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors

set -euo pipefail
cd "$(dirname "$0")/.."

function task() {
        printf '\033[36m[TASK] \033[0m%s\n' "$*"
}

function fail() {
        printf '\033[31m[FAIL] \033[0m%s\n' "$*"
        exit 1
}

function ok() {
        printf '\033[32m[ OK ] \033[0m%s\n' "$*"
}

function check_git_clean() {
        if ! git rev-parse --is-inside-work-tree >/dev/null ; then
                return 0
        fi
        output=$(git status --porcelain --untracked-files=all) || return 1
        if echo "$output" | grep '^.[^ ]' >/dev/null ; then
                return 1
        fi
        return 0
}

TMPDIR=$(mktemp -d)

task "The Git directory must be clean"
check_git_clean || fail "Your Git tree is dirty. Please stage or stash the changes."

task "The program must build"
BUILDDIR="$TMPDIR/build"
mkdir -p "$BUILDDIR"
meson setup "$BUILDDIR" || fail "meson.build does not compute."
ninja -C "$BUILDDIR" || fail "The program does not compile."

task "GLib files must be valid"
glib-compile-schemas --targetdir "$BUILDDIR" --strict data || fail "glib-compile-schema fails"
desktop-file-validate "$BUILDDIR/data/es.danirod.Cartero.desktop" || fail "desktop-file-validate fails"
appstreamcli validate --no-net "$BUILDDIR/data/es.danirod.Cartero.metainfo.xml" || fail "appstreamcli validate fails"

task "The program must be valid"
export GSETTINGS_SCHEMA_DIR="$BUILDDIR"
cargo test --workspace
cargo clippy --workspace

task "The code has to be formatted and linted"
cargo fmt --check --all || fail "Please use 'cargo fmt' to format the files"
BLUEPRINT=$(which blueprint-compiler 2>/dev/null || echo "subprojects/blueprint-compiler/blueprint-compiler.py")
$BLUEPRINT format data/ui || fail "Please use '$BLUEPRINT format' to lint the blueprint files"

task "Translations must be up to date"
sed -e '/^\s*#.*/d' -e '/^\s*$/d' < po/LINGUAS | LC_ALL=C sort -c || fail "Please tidy po/LINGUAS alphabetically, it prevents Git conflicts!"
MSGDIFF="$TMPDIR/pocheck"
mkdir -p "$MSGDIFF"
build-aux/check-po.sh > "$MSGDIFF/old"
ninja -C "$BUILDDIR" cartero-pot
ninja -C "$BUILDDIR" cartero-update-po
build-aux/check-po.sh > "$MSGDIFF/new"
diff -u "$MSGDIFF/old" "$MSGDIFF/new" || fail "The .po files are outdated, please update them!"
git checkout -- po

task "The licenses must be valid"
cargo deny --version || fail "Please install cargo-deny!"
cargo deny --workspace check licenses || fail "The depencies do not meet the licensing requirements!"
reuse --version || fail "Please install reuse"
reuse lint || fail "Please check the output of 'reuse lint' and address the issues"

task "The NEWS file must be updated"
VENVDIR="$TMPDIR/venv"
mkdir -p "$VENVDIR"
python -m venv "$VENVDIR"
$VENVDIR/bin/python -m pip install babel lxml mdformat
PATH="$VENVDIR/bin:$PATH" build-aux/news-update.py
git diff --quiet || fail "NEWS file is not updated, please run build-aux/news-update.py"
git checkout -- .

ok "Everything fine"
