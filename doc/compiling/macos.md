# Compiling on macOS

The process is not very clean at the moment.

1. [Homebrew](https://brew.sh) should be installed.
1. Install Rust (suggestion: rustup)
1. The following dependencies should be available (suggestion: Homebrew):
   `meson pkg-config gtk4 gtksourceview5 desktop-file-utils pygobject3 libadwaita adwaita-icon-theme shared-mime-info`.
1. You can compile the application using Meson like on any other platform,
   as long as the dependencies are accessible to Meson.

## Packaging as an .app bundle

On macOS, graphical applications usually use the application bundle format.
It is a directory whose name ends with .app and follows some specific tree
requirements.

Anyway, the point is that you probably want Meson to compile an .app. You
currently can do this if you do the following three things:

- You set the prefix to `/`.
- You set the destdir to the location where you want to build an app bundle.
- You enable the `macos-app` meson option.

So:

```bash
meson setup --prefix=/ -Dmacos-app=enabled build
DESTDIR=$PWD/output ninja -C build install
```

Gotchas:

* Currently you should always set DESTDIR to an empty directory. Failure to follow
  this rule may cause additional files to be copied to the .app file. This could be
  fixed in the future.
* If you rebuild or reinstall, sometimes the process may fail randomly. If that's
  the case, remember that it's easier to `rm -rf` the DESTDIR to clean it before
  regenerating the .app.

The application will not be signed by default. If you don't sign the application,
it will probably won't be able to get notarized. If it's not notarized, it is
more difficult to distribute the app.

The install script can automatically sign the application for you if you provide
the valid sign identity. Once you have a valid Apple Developer ID installed on
your system, run `security find-identity -v -p codesigning` to get its hash
(a long hexadecimal string).

Use the `macos-codesign-identity` meson option when setting up the project. This
will cause the install script to use the given identity to sign the application:

```bash
meson setup --prefix=/ -Dmacos-app=enabled -Dmacos-codesign-identity='ABCABC...' build
DESTDIR=$PWD/output ninja -C build install
```

## Packaging as a .dmg

You can also generate a .dmg installer.

It is required to install [`create-dmg`](https://github.com/create-dmg/create-dmg).
You can install it via Homebrew.

The install script will first generate an .app using the steps given before, then
pack it into a .dmg file. If you provide a valid keychain profile for notarizing the
app, it will also send it to Apple for notarization and staple the notarization
output. A notarized application is trusted and therefore can safely be distributed
to other computers without having to fallback to workarounds to run the app.

The Meson options of interest are:

* `macos-dmg`: it has to be enabled.
* `macos-notary-profile`: the keychain profile to use. This is equivalent to using the `-p`
  option when calling `xcrun notarize`.

Note that you can omit the `macos-notary-profile` option. If you don't pass this option,
the .dmg will not be notarized, which will limit the distribution of the app, because
macOS nowadays requires every app to be notarized in order to successfully run. You can
use workarounds such as calling `xattr -c` on an .app bundle to remove the quarantine bits,
but these are not future-proof because forcing developers to pay a yearly fee to let software
exist for your operating system is a very interesting business model for Apple.

You have to also provide a valid codesign identity via the `macos-codesign-identity`, or
the notarization process will fail. If you enable the `macos-dmg` option, the `macos-app`
option is inferred as true even if you don't enable it.

To build a .dmg without notarization:

```bash
meson setup --prefix=/ -Dmacos-dmg=enabled build
DESTDIR=$PWD/output ninja -C build install
```

To build a .dmg with notarization:

```bash
meson setup --prefix=/ -Dmacos-dmg=enabled -Dmacos-codesign-identity='ABCABC...' -Dmacos-notary-profile='profile' build
DESTDIR=$PWD/output ninja -C build install
```
