# Compiling on Windows

As a Rust application, compiling Rust on Windows should be easy, or even
cross-compiling from a different OS with the proper cross-toolchain installed.

However, to run Cartero you'll need the GTK runtime available. Currently,
GTK applications for Windows solve this by vendoring third party dependencies
such as:

- The .dll files for every shared library the program depends on.
- Any datafiles required by those dependencies (GtkSourceView color schemes,
  GLib 2.0 schemas for GTK...)
- Default icon themes so that the default app theme works correctly.

If you have any GTK application on Windows such as GIMP or Inkscape, open the
install directory and you'll see what I mean. The executable lives under
bin\gimp.exe or bin\inkscape.exe, but there are a lot of GTK and dependency
DLLs in bin\\*.dll, and there will be lots of files in share\\glib-2.0 and
probably share\\icons or share\\themes.

There are two ways to get an updated GTK runtime on Windows:

- Use MSYS2, which provides these dependencies linked against the GNU (x64)
  or the GNU+LLVM (ARM64) toolchain. For example, [mingw-gtk4].
- Use [gvsbuild], which provides these dependencies linked against the
  MSVC toolchain (Microsoft Visual Studio). Only x64 is currently supported,
  however.

I haven't tested this a lot, but I assume that you shouldn't mix sources.
If you are going to use the MSYS2 toolchain, you should compile Cartero with
either the [pc-windows-gnu] (x64) or the [pc-windows-gnullvm] (arm64) Rust
toolchains; or use a [pc-windows-msvc] (x64, arm64) if you are using gvsbuild.

[mingw-gtk4]: https://packages.msys2.org/base/mingw-w64-gtk4
[gvsbuild]: https://github.com/wingtk/gvsbuild
[pc-windows-gnu]: https://doc.rust-lang.org/nightly/rustc/platform-support/windows-gnullvm.html
[pc-windows-gnullvm]: https://doc.rust-lang.org/nightly/rustc/platform-support/windows-gnullvm.html
[pc-windows-msvc]: https://doc.rust-lang.org/nightly/rustc/platform-support/windows-msvc.html

## Recommended workflow: MSYS2

Currently Cartero is known to build under both UCRT64 for x64 and CLANGARM64
for arm64. Here, by _build_ I mean both using the Rust compiler to generate
`cartero.exe`, and then successfully downloading and vendoring the whole
GTK runtime.

> **NOTE**: Other MSYS platforms such as CLANG64 or MINGW64 should work.
However, the advantage of UCRT64 is that it compiles against the CRT provided
by Windows 10 and Windows 11. MINGW64 depends on the GNU CRT and therefore
it will require additional .dll files to work.

> **NOTE**: Even though Cartero is known to compile for arm64, this has only
been tested through a CI environment. I don't have access to a Windows 11 ARM
computer so I have never seen Cartero running on Windows 11 ARM. If you have
the valid hardware and want to compile and report, that would be great. If
Cartero doesn't run properly on Windows 11 ARM, you should download Cartero
for x64 anwyay and run it through the x64 emulation present in Windows 11 ARM.

The process is very similar to the instructions described in
the [gtk-rs book][win] with the GNU toolchain.

1. Install MSYS2 from https://www.msys2.org.
1. Install Rust (suggestion: rustup).
   - rustup does not carry the `aarch64-pc-windows-gnullvm` toolchain because
     it is a Tier 2 platform. If you don't want to compile the toolchain on
     your own, you should just install the `mingw-w64-clang-aarch64-gtk4`
     package from within MSYS-CLANGARM64 and call it a day.

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

[win]: https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html

## Creating a proper distribution

Note that the application that you just built with MSYS2 will only work when
working inside MSYS2, because everything is in the shell env: DLLs, additional
GTK schemas...

However, if you plan to distribute the compiled artifacts, or just want to run
the application outside of MSYS2, you have to create a **distribution**, and
vendor every dependency, including DLL files and other data files.

There is a Meson option called `win32-bundle`. Enable this option and when
the `install` target is called using a custom DESTDIR, it will vendor every
required dependency and their datafiles.

Therefore, compiling a proper distribution requires running something similar
to the following:

```sh
meson setup build -Ddecorations=no-csd -Dwin32-bundle=enabled --prefix=\\
DESTDIR=$PWD/win32 ninja -C build install
```

**Note that the prefix will have to be given as a Windows path rather than a
POSIX path**. Therefore, you'll have to pass `\\` rather than `/` if you run
the command directly on the shell. If you use a shell script to execute the
commands, maybe you can pass `/` if meson thinks is running under a POSIX
environment. Failure to comply with this will not cause damage, but probably
Meson will ignore the prefix you provide and use the default one anyway,
creating nested directories.

This will create a `win32` directory where Cartero will be "installed",
copying bin\cartero.exe, every datafile of Cartero itself, and also vendor
every required library, additional gettext locale file, image loader, icons
and other support files.

## Additional actions

Some actions are present in the Meson project because they are used as part of
the official releng made by the Cartero team. While you can use these workflows
on your own, these are private and they can change or be removed at any time if
we find a better way to bundle our official release.

### Signing the application

Modern Windows versions present a scary SmartScreen warning if you try to
run a program that has not been digitally signed. _This is, of course, very
safe because as we all know, hackers can't steal codesign certificates and
thus cannot sign their malware to bypass SmartScreen. Very smart!_

Note that if you want to distribute the installer or the compiled version,
you should have a valid certificate to sign the application. Otherwise, when
running the program on a different system, a warning will be presented, that
has to be accepted to run the application.

The Meson buildscript has an option called `win32-sign-subject`. If the option
is defined, the `signtool.exe` program will be called after bundling the
application if the `win32-bundle` or `win32-installer` options are enabled,
and after creating the installer, if the `win32-installer` option is enabled.

The `win32-sign-subject` is a string option that maps to the `/n` parameter
provided to signtool.exe. The signtool call is configured to dual-sign the
given executables both with a SHA-1 and a SHA-256 signature. The timestamp
server is already set to the one from Certum. Use the meson option to provide
the subject name of the certificate, and be ready to provide a PIN if needed.
(You may have to enter the PIN up to 4 times, because it's calling signtool.exe
up to 4 times depending on whether the installer is enabled or not.

```sh
meson setup build -Ddecorations=no-csd -Dwin32-installer=enabled -Dwin32-sign-subject="John Doe" --prefix=\\
DESTDIR=$PWD/win32 ninja -C build install
```

This step is probably too coupled to the [release engineering](../releng.md)
process. Official Cartero binary distribution files for Windows are signed
with a Certum open source certificate, which explains why the timestamp
server is the one from Certum.

### Creating an installer

Use the `win32-installer` Meson option to generate an .iss file compatible
with [Inno Setup][innosetup], to produce a valid installer. If InnoSetup is
installed and ISCC.exe is found in your PATH, the installer will be
automatically built in the DESTDIR.

```sh
meson setup build -Ddecorations=no-csd -Dwin32-installer=enabled --prefix=\\
DESTDIR=$PWD/win32 ninja -C build install
```

Signing an installer does not sign the executable files installed by it, so
you should also sign cartero.exe if you plan on distributing the installer
to other systems.

[innosetup]: https://jrsoftware.org/isinfo.php

## Experimental workflows

The MSYS2 workflow is being used because it is known to work.

I don't spend a lot of time trying to improve things because I barely use
Microsoft Windows for programming.

If you regularly develop for Rust and/or GTK on a Windows environment and
you have a way to enhance the process, start a discussion or send a pull
request. If it works and it makes things better, it will be accepted.
