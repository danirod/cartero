# Hacking Cartero

Some additional instructions for people interested in running cartero.

## "Meson sucks, stick with Cargo"

I don't disagree with this statement, in fact. I fail to see exactly how is
Meson better than CMake when it does the same tasks, but sometimes worse, since
Meson is not as flexible and extensible as CMake is, and this is seen as how
fragile is the integration between Meson and Rust.

Meson is as opinionated on how to build a program as Cargo is. Meson will call
Cargo when it is time to build the executable file by just issuing the `cargo
build` external command, but the build script still has to copy a lot of files
so that both tools can coexist without shilling at each other publicly on your
terminal.

This project is making use of it because of two reasons:

* Because there is actually more than Rust for this project and Meson knows
  how to compile the resource bundles, update the locales and generate the
  desktop files for Cartero.

* Because it ticks a checkbox.

To be honest, if you are not a try-hard GNOME developer, you will find casually
hacking applications based on the GNOME stack difficult and cumbersome because
some of the following reasons mean that "installing" the app, or at least the
data files (/usr/share), is necessary to run the program.

* Many applications will make use of the GSettings framework to load and store
  application settings, and GSettings require the GSchemas to be installed
  globally in order to use them.

* Some features, such as application notifications, may not work at all unless
  the application is installed and thus recognised by the GNOME notifications
  framework.

* Some applications insist on using absolute and hardcoded locations when
  loading translation files and GResource bundles, such as always trying to load
  from $prefix/share. Therefore, unless the application is installed, the file
  system hierarchy will not be fully set.

A valid answer to these complains would be "just compile and debug a Flatpak",
but this will not be the taste of some people and I understand that.

My suggestion however is to just use a local installdir such as $PWD/install,
and just run `ninja install` into your prefix as part of the compilation
process. This doesn't require root privileges (DO NOT SUDO WHILE TESTING OR
RUNNING CARTERO FOR GOD'S SAKE) and it can be easily cleaned with `rm -rf
install` later.

### Alternative build script

Precisely, that's what my alternative build scripts do.

**build-aux/meson-build.sh** is a script that doesn't do anything different than
what the README.md file indicates, but it automates it in order to do it very
quick. So running **build-aux/meson-build.sh** will run `meson setup` and `ninja
&& ninja install` for you, using the following paths:

* It will build the application into $PWD/build.
* It will install the application to $PWD/install.

So a quick workflow for hacking Cartero while using the script would be:

```sh
build-aux/meson-build.sh && install/bin/cartero
```

As you probably have noticed, Meson sucks, and sometimes it will not detect that
the GResource file has to be recompiled, despite my attempts to make sure that
every target in my meson.build files has dependencies properly set.

If you notice while running meson-build.sh that the application crashes or does
not detect the changes that you made to the user interface files, my suggestion
is to just run `rm -rf build/data` and recompile. Meson should see that the
resource file is missing and it should create it from scratch.

### But I want to use cargo build

I know, and it makes a lot of sense, because chances are that you are using a
text editor or IDE (this includes GNOME Builder) with some dveloper tools such
as rust-analyzer, and this directory will probably want to use `target` to do
its stuff and provide completions.

As long as you have the dependencies in your system (GTK, SourceView5...), you
should be able to just `cargo build` the application and **it has to compile**.
If `cargo build` does not work, that's a bug.

However, since you need the resource bundle and the translation files, `cargo
run` will not work unless you place them in your target directory as well, as I
tried to explain above.

The good thing is that Cargo actually builds your application into a
subdirectory of `target`, such as `target/debug` or `target/release`. In this
case, the `debug` and `release` directories act as the bindir for cartero, so
all you have to do is to create a `target/share` directory, place all the
datafiles in `target/share`, and that's it.

**And that is exactly what build-aux/cargo-build.sh** does. So if you want to
use `cargo build` and `cargo run`, just use `build-aux/cargo-build.sh`, which
calls `cargo build` for you, and additionally it runs the Meson targets required
to craft a valid pkgdatadir. It will then proceed to deploy them into
`target/share`, which will act as the datadir for the app when you run it with
`cargo build`. The workflow will be:

```sh
build-aux/cargo-build.sh && cargo run
```

Just the same as above, if you notice that your user interface files are not
updating, just run `rm -rf build/data` and try to compile the application again.

### Two CARGO_HOMEs?

One thing to point out is that Meson uses its own vendored CARGO_HOME directory
and it compiles into a different target directory inside build. Therefore, if
you mix both pipelines, you will probably see more often Cargo downloading and
compiling dependencies.

I think that the vendoring process made by Meson is correct and useful and thus
I do not see this as a drawback. I'm just pointing this out because waiting for
Cargo to compile dependencies suck and mixing `cargo build` with `meson` will
make you wait more often.

### Absolute directories

Just as a note: Cartero does not make use of absolute paths. You should be able
to run the application from any location, as long as there is a separate bindir
and datadir. I placed this requirement for the future, since I want a Microsoft
Windows installer to exist and I cannot just run cargo build inside the
installer. (Or can I?!)
