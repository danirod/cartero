# Compiling on Windows

Cartero has to build for Windows because Cartero is available for Windows.
However, support for Windows is very partial because is not a platform where
many development time is being spent.

We currently only support MSYS2-UCRT. Switching to [gvsbuild] would be nice and
compiling using the MSVC backend for Rust would make the build times faster,
but is currently not a top priority. If this affects you and you have the
knowledge, start an issue or open a PR and it will be gladly accepted if it
improves things.

The process is very similar to the instructions described in
the [gtk-rs book][win] with the GNU toolchain.

1. Install Rust (suggestion: rustup)
1. Install MSYS2 from https://www.msys2.org.

MSYS2 has multiple subsystems. **I suggest UCRT64 because it works out of the
box with Windows 10 and Windows 11. The official binaries for Cartero are
built in UCRT64 mode**. You can build using CLANG64 or MINGW64 too and it
should work, but I don't test it very often so I cannot guarantee. 

Make sure that your Rust compiler is added to the MSYS PATH. For instance,
`export PATH=/c/Users/[username]/.cargo/bin:$PATH` on the .bashrc for your
shell, or run the command before starting the compile session.

Then install the dependencies. You can use `pacboy` from the `pactoys` package:

```sh
pacman -S pactoys
pacboy -S blueprint-compiler desktop-file-utils gcc gettext gtk4 gtksourceview5 libadwaita librsvg libxml2 meson pkgconf
```

You can also install manually the packages without extra tools, but make sure
that you install the versions appropiate for your MSYS2 version, so use the
`$MINGW_PACKAGE_PREFIX` when installing the dependencies:

   * `${MINGW_PACKAGE_PREFIX}-blueprint-compiler`
   * `${MINGW_PACKAGE_PREFIX}-desktop-file-utils`
   * `${MINGW_PACKAGE_PREFIX}-gcc`
   * `${MINGW_PACKAGE_PREFIX}-gettext`
   * `${MINGW_PACKAGE_PREFIX}-gtk4`
   * `${MINGW_PACKAGE_PREFIX}-gtksourceview5`
   * `${MINGW_PACKAGE_PREFIX}-libadwaita`
   * `${MINGW_PACKAGE_PREFIX}-librsvg`
   * `${MINGW_PACKAGE_PREFIX}-libxml2`
   * `${MINGW_PACKAGE_PREFIX}-meson`
   * `${MINGW_PACKAGE_PREFIX}-pkgconf`

Then proceed to compile using the standard Meson instructions:

```sh
meson setup build
ninja -C build
```

My suggestion is to disable client side decorations when compiling for
Windows. This will disable the combined "title bar" + "tool bar" in Cartero.
Unfortunately, on Windows the CSD for GTK still have some odd issues, such
as not integrating with Aero Snap or the Windows 11 automatic desktop layouts.
You can disable CSD when setting up the Meson project by using the appropiate
Meson options:

```sh
meson setup build -Ddecorations=no-csd
```

## Creating a proper distribution

Note that the application that you just built with MSYS2 will only work when
working inside MSYS2, because everything is in the shell env: DLLs, additional
GTK schemas...

However, if you plan to distribute the compiled artifacts, or just want to run
the application outside of MSYS2, you have to create a **distribution**, and
vendor every dependency, including DLL files and other data files.

Calling the `install` target when using Meson on Windows will automatically
trigger the `packaging/win32/dependencies.py` script. You should review what
it does if you want to learn more or if you want to do things manually.

Therefore, compiling a proper distribution requires running something similar
to the following:

```sh
meson setup build -Ddecorations=no-csd --prefix=/
DESTDIR=$PWD/win32 ninja -C build install
```

This will create a `win32` directory where Cartero will be "installed",
copying bin\cartero.exe, every datafile of Cartero itself, and also vendor
every required library, additional gettext locale file, image loader, icons
and other support files.

There will even be an .iss file if you want to create your own installer.
You can use [InnoSetup][innosetup] to compile the installer for convenience.

Note that if you want to distribute the installer or the compiled version,
you should have a valid certificate to sign the application. Windows these
days is very picky when it comes to running unsigned code and will probably
render a SmartScreen alert if you try to run an unsigned executable or
installer.

[innosetup]: https://jrsoftware.org/isinfo.php
[gvsbuild]: https://github.com/wingtk/gvsbuild
[win]: https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html