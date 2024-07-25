# Compilation for macOS

The process is in early stages and it will surely be improved in the future.
Things to evaluate: whether to do cross-compilation or compile from macOS. There
are toolchains in rustup, but since there are a lot of system libraries in
place, the question is whether the process will work.

At the moment, just use macOS.

1. [Homebrew](https://brew.sh) should be installed.
2. Install Rust (suggestion: rustup)
3. The following packages should be installed via Homebrew. They have a lot of
   dependencies, so take your time. (Note: you may need to install `svn` if you
   build these packages from source).
   * `meson`
   * `gtk4`
   * `gtksourceview5`
   * `desktop-file-utils`
   * `pygobject3`
   * `libadwaita`
   * `adwaita-icon-theme`
4. Before compiling, you have to export the following environment variable,
   `export GETTEXT_DIR=$(brew --prefix)/opt/gettext`, so that it can actually
   pick your gettext library.
5. Once you have the source code, you should be able to compile it using the
   same instructions in README.md.

## Separate Homebrew

To avoid installing a lot of dependencies in your system-wide Homebrew
installation, you can have a separate Homebrew installation for building this
application.

```sh
git clone https://github.com/Homebrew/brew homebrew
export PATH=homebrew/bin:$PATH
```

In fact, when making changes to this file, it should be done from a clean
environment without system-wide Homebrew and only a local Homebrew installation,
in order to avoid false positives due to dependencies not in this list that were
accidentally installed.
