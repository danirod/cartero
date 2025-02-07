# Compiling on macOS

The process is not very clean at the moment.

1. [Homebrew](https://brew.sh) should be installed.
1. Install Rust (suggestion: rustup)
1. The following packages should be installed via Homebrew. They have a lot of
   dependencies, so take your time. (Note: if the packages are built from source,
   you may need to install `svn` -- svn is not required when bottles are used).
   * `meson`
   * `gtk4`
   * `gtksourceview5`
   * `desktop-file-utils`
   * `pygobject3`
   * `libadwaita`
   * `adwaita-icon-theme`
   * `shared-mime-info`
1. To automatically build an app, use the script `build-aux/macos-build.sh`. As
   long as you have all the dependencies installed, it will compile an application
   into `build/cartero-darwin`. Use `build-aux/macos-build.sh devel` to build a
   development version and `build-aux/macos-build.sh stable` to build a stable
   version.
1. If you want to compile manually (for instance, if you are going to actually
   _develop_ on a Mac), before compiling, you have to export the following
   environment variable: `export GETTEXT_DIR=$(brew --prefix)/opt/gettext`, so
   that it can actually pick your gettext library.
1. To compile the application manually, refer to README.md. Specifically, both
   `cargo build` and `build-aux/cargo-build.sh` should run as long as you have
   all the dependencies.
