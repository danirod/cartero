# Compilation for Windows

The process is in early stages and it will surely be improved in the future.
Things to evaluate: whether to do cross-compilation or compile from Windows.
There are toolchains in rustup, but since there are a lot of system libraries
in place, the question is whether the process will work.

At the moment, just use Windows.

Anyway. The process closely follows the instructions in the [gtk-rs book][win].
I'm using the MSYS2 and GNU toolchain, I haven't tried with MSVC, and, to be
honest, I don't want to.

1. Install Rust (suggestion: rustup)
2. Install MSYS2 from https://www.msys2.org.
3. You will use MSYS2 MINGW64. Do not use neither MSYS2 CLANG64 nor MSYS2
   UCRT64.
4. Make sure that your Rust compiler is added to the MSYS PATH. For instance,
   `export PATH=/c/Users/[username]/.cargo/bin:$PATH`.
5. The list of packages that you should install include:
   * `mingw-w64-x86_64-desktop-file-utils`
   * `mingw-w64-x86_64-gcc`
   * `mingw-w64-x86_64-gettext`
   * `mingw-w64-x86_64-gtk4`
   * `mingw-w64-x86_64-gtksourceview5`
   * `mingw-w64-x86_64-libxml2`
   * `mingw-w64-x86_64-librsvg`
   * `mingw-w64-x86_64-meson`
   * `mingw-w64-x86_64-pkgconf`
   * `meson`
6. Once the repository is cloned, you should be able to compile from inside a
MSYS2 shell. Instructions in the README.md file apply here as well.

[win]: https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html
