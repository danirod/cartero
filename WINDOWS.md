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
3. You are encouraged to use MSYS UCRT64, since UCRT is preinstalled on Windows
   10 and Windows 11, and thus will not require so many dependencies. Either way,
   you can also use MINGW64 and probably CLANG64 as well.
4. Make sure that your Rust compiler is added to the MSYS PATH. For instance,
   `export PATH=/c/Users/[username]/.cargo/bin:$PATH`.

Then, you can either use `build-aux/msys-build.sh` in order to automatically build
the application. **This is the recommended step, and it's the one that will be
faster for you**, because it will already have all the dependencies packaged.

If you still want to do it manually you can do the following extra steps:

1. The list of packages that you should install include:
   * `${MINGW_PACKAGE_PREFIX}-desktop-file-utils`
   * `${MINGW_PACKAGE_PREFIX}-gcc`
   * `${MINGW_PACKAGE_PREFIX}-gettext`
   * `${MINGW_PACKAGE_PREFIX}-gtk4`
   * `${MINGW_PACKAGE_PREFIX}-gtksourceview5`
   * `${MINGW_PACKAGE_PREFIX}-libxml2`
   * `${MINGW_PACKAGE_PREFIX}-librsvg`
   * `${MINGW_PACKAGE_PREFIX}-meson`
   * `${MINGW_PACKAGE_PREFIX}-pkgconf`
   * `${MINGW_PACKAGE_PREFIX}-libadwaita`
   * `${MINGW_PACKAGE_PREFIX}-blueprint-compiler`
   * `meson`
2. Once the repository is cloned, you should be able to compile from inside a
MSYS2 shell using the same instructions that are present in README.md.
3. If you plan on distributing the application, remember to pack all the DLLs.

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
