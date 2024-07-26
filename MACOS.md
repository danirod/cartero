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
   * `shared-mime-info`
4. To automatically build an app, use the script `build-aux/macos-build.sh`. As
   long as you have all the dependencies installed, it will spit an application
   into `build/cartero-darwin`. Use `build-aux/macos-build.sh devel` to build a
   development version and `build-aux/macos-build.sh stable` to build a stable
   version.
5. If you want to compile manually (for instance, if you are going to actually
   _develop_ on a Mac), before compiling, you have to export the following
   environment variable: `export GETTEXT_DIR=$(brew --prefix)/opt/gettext`, so
   that it can actually pick your gettext library.
6. To compile the application manually, refer to README.md. Specifically, both
   `cargo build` and `build-aux/cargo-build.sh` should run as long as you have
   all the dependencies.
