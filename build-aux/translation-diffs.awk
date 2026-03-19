#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors

# A probably too complex awk script that parses a git-diff for a .po file that
# checks whether strings have been translateed in that .po for that diff.
# This was previously a set of greps, but it became too complex because of the
# edge cases.
#
# I have never written an awk script before, so this file is heavily commented
# because I am scared as fuck. If you know awk better than me, you are free to
# improve this file. The following large comment will describe the idea of this
# file, and also help me understand what I am doing.
#
# The idea is that this awk script will parse the incoming git-diff patch
# and `exit 0` when it finds a valid string that qualifies as a translation,
# as part of the news-translations.sh script, in order to properly list the
# updated languages in the NEWS.md file for every version.
#
# So I will take the incoming git-diff patch, and find any lines that adds
# or update a translation. These are the ones that begin with `msgstr`, so the
# token to find is `+msgstr` (mind the plus sign that is part of the diff).
#
# But not every msgstr line qualifies. We have to disregard the following:
#
# 1. The translation for the copyright line, the one that begins with
#    "Copyright © 2024-", because I update those every year and it would
#    acknowledge an update for every translation on the first release of the
#    year.
#
# 2. Fuzzy translations. If you find a +msgstr string, but there is a #, fuzzy
#    in the same paragraph, you have to disregard the translated string because
#    it has not been reviewed by an human and it is probably just gettext
#    incorrectly guessing a line.
#
# 3. Empty strings. Because not every string has to be translated, so it is
#    perfectly valid to find `+msgstr ""` in the diff: that would mean that
#    either the string is new, or that the locale is new and not every string
#    is translated yet. But life is not easy. Sometimes long strings are split
#    into multiple lines, and the first line of the multiline translation IS
#    actually a `msgstr ""`. So I cannot just drop empty strings, I have to
#    check if the next line contains a non-empty string and only disregard
#    the translation if it is the end of the paragraph.
#
# 4. The metadata translation. Every .po file has this as the first string.
#    It is not a "real" translation, so it should not be considered.
#
# Way of doing things: setup some variables in the BEGIN, update the state
# as I scan my lines of interest. Empty lines in gettext separate translations,
# so if I find an empty line, I early-return if I found a valid string, or I
# reset the state and continue scanning if I haven't found a valid one yet.
#
# Good luck from now on.

# Init my variables.
BEGIN {
    fuzzy = 0
    found = 0
    empty_string = 0 # state machine that remembers empty strings to choose
    metadata = 0
}

# Empty line, early return if found, or reset the state otherwise.
/^\s*[\+\-]?\s*$/ {
    if (found) {
        exit 0
    }
    fuzzy = 0
    found = 0
    empty_string = 0
    metadata = 0
    next
}

# Found the metadata string, so ignore.
/^[\+ ]msgid ""/ {
    metadata = 1
    next
}

# Fuzzy string incoming, ignore this block.
/^[\+ ]#, fuzzy/ {
    fuzzy=1
    next
}

# Adds a copyright, ignore this line.
/^\+msgstr "© 2024-/ {
    found = 0
    next
}

# Found an empty string. Interesting, as long as the string is not fuzzy.
/^\+msgstr(\[[0-9]+\])? ""/ {
    empty_string = 1
    next
}

# Found any other kind of +msgstr string, so found!
/^\+msgstr/ {
    if (!fuzzy && !metadata) {
        found = 1
    }
    next
}

# This one is metadata too and it often comes in diffs.
/^[\+ ]"POT-Creation-Date:/ {
    metadata = 1
    next
}

# This one also seems to catch a lot of false positives.
/^[\+ ]"Project-Id-Version:/ {
    metadata = 1
    next
}

# A sudden string in a separate line qualifies as found if conditions apply.
/^\+\s*"[^"]+"/ {
    if (!fuzzy && empty_string && !metadata) {
        found = 1
    }
    next
}

# End of file reached, If the translated string comes last, there will be no
# paragraphs, so I cannot just exit 1, I have to check again for found.
END {
    if (found) {
        exit 0
    } else {
        exit 1
    }
}