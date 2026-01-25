# Compiling on macOS

The process is not very clean at the moment.

1. [Homebrew](https://brew.sh) should be installed.
1. Install Rust (suggestion: rustup)
1. The following dependencies should be available (suggestion: Homebrew):
   `meson pkg-config gtk4 gtksourceview5 desktop-file-utils pygobject3 libadwaita adwaita-icon-theme shared-mime-info`.
1. If you are using Homebrew, remember to export an env var called
   `GETTEXT_DIR` to override the default system gettext and force the
   compile process to use the Homebrew one:
   `export GETTEXT_DIR=$(brew --prefix)/opt/gettext`
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
