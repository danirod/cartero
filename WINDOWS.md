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
   * `mingw-w64-x86_64-libadwaita`
   * `mingw-w64-x86_64-blueprint-compiler`
   * `meson`
6. Once the repository is cloned, you should be able to compile from inside a
MSYS2 shell. Instructions in the README.md file apply here as well.
7. If you plan on distributing the application, remember to pack all the DLLs.

# Including dependencies

Note that the application that you just build with MSYS2 will work because you
previously installed all the dependencies using MSYS2. Thus, as long as
C:\msys64\mingw64\bin is in your PATH, Windows will be able to find the DLLs for
stuff like GTK, GtkSourceView and similar.

However, if you plan on distributing the application, the target system may not
have these DLLs installed. Instead of just asking your users to install MSYS,
you should distribute the dynamic libraries that are required to run the
application with the bindir.

I'm sure there is a better process, but for the time being, just use ldd inside
MSYS2 to see which DLLs are required to run cartero.exe and filter those that
are in /mingw64/bin, like so:

```sh
ldd install/bin/cartero.exe | grep '/mingw64/bin'
```

Then, to copy these dependencies, you can use this invocation:

```sh
for dep in $(ldd install/bin/cartero.exe |grep '/mingw64/bin' | awk '{ print $1 }'); do cp /mingw64/bin/$dep install/bin; done
```

If you run ldd again, you should verify that now all these libraries are taken
from the install directory, since Windows if there are multiple DLLs with the
same name, Windows gives priority to the one in the same directory as the .exe
file.

[win]: https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html
