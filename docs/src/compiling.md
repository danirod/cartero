# Compiling from sources

Currently, to build the application you'll have to make sure that the required
libraries are installed on your system.

- glib >= 2.72
- gtk >= 4.14
- gtksourceview >= 5.4
- libadwaita >= 1.5
- openssl >= 1.0

For a successful build, will also need the following packages installed in your system: **meson**, **ninja**, **rust** and **gettext**.

Then use the following commands to build and install the application

```sh
meson setup build
ninja -C build
ninja -C build install
```

To avoid installing system-wide the application, you can use a prefix:

```sh
meson setup build --prefix=$HOME/usr
ninja -C build
ninja -C build install
```
