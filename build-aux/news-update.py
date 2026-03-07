#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors
#
# Rebuilds NEWS and release notes

import os
import mdformat
from babel import Locale
from io import StringIO, BytesIO
from lxml import etree
from pathlib import Path

os.chdir(Path(__file__).resolve().parent.parent)

# Utils.
def read_changelog(base, name):
    path = base.joinpath(name)
    if path.exists():
        return path.read_text().splitlines()

def read_file(base, name):
    path = base.joinpath(name)
    if path.exists():
        return path.read_text().rstrip("\r\n")

def write_news_block(f, title, lines):
    if not lines:
        return
    f.write(f'### {title}\n\n')
    for line in lines:
        f.write(f'* {line}\n')
    f.write('\n')

def write_xml_block(node, title, lines):
    if not lines:
        return
    par = etree.Element('p')
    par.text = title
    node.append(par)
    ul = etree.Element('ul')
    for line in lines:
        li = etree.Element('li')
        li.text = line
        ul.append(li)
    node.append(ul)

# Parse the NEWS.d directory to build the NEWS model.
news = {}
versions = [dir for dir in Path('NEWS.d').iterdir() if dir.is_dir()]
for version in versions:
    if version.name == 'unreleased':
        continue
    news[version.name] = {
        'notes': read_file(version, 'notes'),
        'released': read_file(version, 'release-date'),
        'added': read_changelog(version, 'added'),
        'changed': read_changelog(version, 'changed'),
        'fixed': read_changelog(version, 'fixed'),
        'translated': sorted(read_changelog(version, 'translated')),
    }
versions = sorted(news.keys(), reverse=True)

# Write NEWS.md
notes = StringIO()
notes.write('# News file for Cartero\n\n')
notes.write('These are the user-visible changes noticeable within Cartero.\n\n')
for version in versions:
    if news[version]['released']:
        notes.write(f"## [{version}] - {news[version]['released']}\n\n")
    else:
        notes.write(f"## [{version}] - unreleased\n\n")
    if news[version]['notes']:
        notes.write(news[version]['notes'])
        notes.write('\n\n')
    write_news_block(notes, 'Added', news[version]['added'])
    write_news_block(notes, 'Changed', news[version]['changed'])
    write_news_block(notes, 'Fixed', news[version]['fixed'])
    if news[version]['translated']:
        write_news_block(notes, 'Translation Updates', [
            Locale.parse(locale).get_display_name('en')
            for locale in news[version]['translated']
            ])
with open('NEWS.md', 'w') as f:
    clean_notes = mdformat.text(notes.getvalue(), options={
        'wrap': 71,
    })
    f.write(clean_notes)

# Update metainfo file.
xml_file = Path('data/es.danirod.Cartero.metainfo.xml.in.in')
parser = etree.XMLParser(remove_comments=False)
tree = etree.parse(xml_file, parser)
releases = etree.Element("releases")
for version in versions:
    if version not in news or not news[version]['released']:
        continue
    release = etree.Element("release", date=news[version]['released'], version=version)
    description = etree.Element("description", translatable="no", translate="no")
    if news[version]['notes']:
        par = etree.Element('p')
        par.text = news[version]['notes']
        description.append(par)
    write_xml_block(description, 'Added:', news[version]['added'])
    write_xml_block(description, 'Changed:', news[version]['changed'])
    write_xml_block(description, 'Fixed:', news[version]['fixed'])
    if news[version]['translated']:
        write_xml_block(description, 'Translation Updates:', [
            Locale.parse(locale).get_display_name('en')
            for locale in news[version]['translated']
            ])
    release.append(description)
    releases.append(release)
tree.find("releases").getparent().replace(tree.find("releases"), releases)
etree.indent(tree, space="  ")
tree.write(xml_file, encoding="utf-8", xml_declaration=True, pretty_print=True)

# As a side-effect, also update changelog.xml.inc:
changelog_inc = Path('src/widgets/changelog.xml.inc')
changelog = tree.find("releases").find("release").find("description")
changelog_io = BytesIO()
for node in changelog:
    next_root = etree.ElementTree(node)
    etree.indent(next_root, space="  ")
    changelog_io.write(etree.tostring(next_root, encoding="UTF-8", pretty_print=True, with_tail=False))
changelog_inc.write_bytes(changelog_io.getvalue())
