# Hacking Cartero

Some additional instructions for people interested in running cartero.

## TL;DR

This project makes use of Meson, whether you like it or not. If you are used
to GNOME app development, you might already like it. Otherwise, you'll have to
accept it.

However, treating this source code workspace as a standard Rust project makes
sense because then you can use autocompletion and other Rust tools. Therefore,
to make things easier, there is an aux script that you can use to quickly rebuild
the application data files (resource bundles, translations, icons, GLib schemas)
and move them to the `target/` directory:

```
build-aux/cargo-build.sh
```

You should check the contents of the script before running it. But it will
precompile parts of the application using meson and the run `cargo build`. You
can then call `cargo r`, `cargo t` or whatever you plan on doing with the code.

## "But Meson sucks, stick with Cargo!"

I don't disagree with this statement, in fact. I feel like Meson is not as
flexible as other build systems like CMake, specially when it comes to creating
and running custom commands. This has the effect of making the integration
between Rust and Meson very fragile.

Meson is an opinionated tool. However, Cargo is also a very opinionated tool.
Meson will invoke Cargo when it is time to build the executable file by just
issuing the `cargo build` external command, but the build script still has to
copy a lot of files to Meson's build directory so that both tools can coexist
without shilling at each other publicly on your terminal.

Why does Cartero use Meson?

* Because there is actually more than Rust for this project and Meson knows
  how to compile the resource bundles, update the locales and generate the
  desktop files for Cartero.

* Because it ticks a checkbox when it comes to aligning to the GNOME guidelines,
  which this project intends to follow, even though Cartero supports a wide
  variety of operating systems and desktop environments.

Due to these things, switching to `build.rs` will not fix things at all, it
will just turn the tables upside down, making Rust code easier to build at the
expense of making every non-Rust resource way more difficult to compile.

## But I want to use cargo build

I know, and it makes a lot of sense, because chances are that you are using a
text editor or IDE (this includes GNOME Builder) with some developer tools such
as rust-analyzer, and this directory will probably want to use `target` to do
its stuff and provide completions.

`cargo build` will just care about compiling the application provided that you
have every system dependency installed (GTK, GtkSourceView...). It doesn't care
about whether the resources or translations have been bundled.

Theferore, `cargo build` has to work anyway. You should be able to just run
`cargo build` in a clean workspace and it has to compile the application.
**If it doesn't work, then that's a bug.**

However, since you need the resource bundle and the translation files, `cargo
run` will not work unless you place them in your target directory as well, as I
tried to explain above.

Cartero will follow the standard UNIX directory standards. Therefore, it expects
to be running inside some kind of `bindir` and by default it will assume that the
datafiles are in `../share`.

In normal circunstances, your bindir will be `/usr/bin` and therefore the datafiles
will be in `/usr/bin/../share => /usr/share`.

However, because Cargo will build the application into a subdirectory inside of
`target`, whether that's `target/debug` or `target/release`, this directory can
act as a valid `bindir`. If the datafiles are placed into `target/share`, then
Cartero has to run.

**And that is exactly what build-aux/cargo-build.sh** does. So if you want to
use `cargo build` and `cargo run`, just use `build-aux/cargo-build.sh`, which
calls `cargo build` for you, and additionally it runs the Meson targets required
to craft a valid pkgdatadir. It will then proceed to deploy them into
`target/share`, which will act as the datadir for the app when you run it with
`cargo build`. The workflow will be:

```sh
build-aux/cargo-build.sh && cargo run
```

If you notice that your user interface files are not updating or your translations
are not being picked up, you can just run `rm -rf build/data` or `rm -rf target/share`
and try again. (If you have to do this a lot, this is probably a bug.)
