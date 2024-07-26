<p align="center"><img src="data/icons/scalable/apps/es.danirod.Cartero.svg" width="256" height="256" alt=""></p>

<h1 align="center">Cartero</h1>
<p align="center">Make HTTP requests and test APIs.</p>

<p align="center"><img src="doc/cartero-screenshot.png" alt="Screenshot of Cartero"></p>

## Motivation

This project exists because there aren't many native graphical HTTP testing
applications / graphical alternatives to cURL that are fully free software, and
I think the world has had enough of Electron / non-native applications that are
anonymously accesible until one day you are forced to create an account and
log in to use just to make some investor happy with their numbers or to chug
some unwanted artificial intelligence at users.

## Download

**Coming soon to Flathub**. I am still figuring out how to deploy to Flathub.
Come back later, or download a bundle file.

You can also download a development version.

|                             | Platform                   | Nightly                            |
|-----------------------------|----------------------------|------------------------------------|
| ![Flatpak](doc/flatpak.png) | Flatpak GNU/Linux, x86_64  | [Nightly][nightly-flatpak-x86_64]  |
| ![Flatpak](doc/flatpak.png) | Flatpak GNU/Linux, aarch64 | [Nightly][nightly-flatpak-aarch64] |
| ![Windows](doc/windows.png) | Windows, x86_64            | [Nightly][nightly-windows-x86_64]  |
| ![macOS](doc/macos.png)     | macOS, Apple Sillicon      | [Nightly][nightly-macos-sillicon]  |
| ![macOS](doc/macos.png)     | macOS, Intel               | [Nightly][nightly-macos-intel]  |

## Building

### Flatpak

Install the runtime:

```sh
flatpak install --user org.gnome.Sdk//46 org.freedesktop.Sdk.Extension.rust-stable//23.08
```

Install the Flatpak, possibly passing the `--user` parameter.

```sh
flatpak-builder --user --install flatpak_app build-aux/es.danirod.Cartero.json
```

You will find Cartero in your application launcher, or you can launch it with
`flatpak run es.danirod.Cartero`.

### Meson

Use this to build the application outside of a Flatpak image. Note that this method should
only be used by power users, to package the application on build scripts for rolling-release
distributions, or for development purposes.

Currently, to build the application you'll have to make sure that the required
libraries are installed on your system.

* glib >= 2.72
* gtk >= 4.14
* gtksourceview >= 5.4
* libadwaita >= 1.5

Make sure that you have Meson in your system. For instance,

```sh
sudo apt install meson
sudo dnf install meson
sudo pacman -S meson
```

Then use the following commands to build and install the application

```sh
meson setup build
ninja -C build
ninja -C build install
```

To avoid installing system-wide the application, you can use a prefix:

```sh
meson setup build --prefix=/usr
ninja -C build
ninja -C build install
```

**If you plan on contributing to the project**, use the development profile.

```sh
meson setup build -Dprofile=development
```

It will also configure a Git hook so that the source code is checked prior to
authoring a Git commit. The hook runs `cargo fmt` to assert that the code is
formatted. Read `hooks/pre-commit.hook` to inspect what the script does.

### Cargo (Rust)

This may be the preferred approach if you use rust-analyzer and tools like
that, because Meson uses a separate target directory, and you might prefer to
use the standard target/ directory.

You can use `cargo build` and `cargo run` to build and run the project
executable, but you will still need to use Meson to build the data files
(GSettings schema files, Gio resource files, gettext translations...).

**You should use `build-aux/cargo-build.sh`**. It wraps `cargo build`, but also
compiles the data files and copies them in `target/share`, so that during
application startup those files can be picked.

### Nix/NixOS

Use this approach to install, build or try cartero on a nixos system. Instructions
assume you're using a flakes nixos system, but you could install it in a regular
nixos system aswell by importing the derivation and adding the appropiate src attribute
on it, note that this may require some manual intervation though.

First of all, add cartero to your flake inputs so you can import the package.

```nix
{
  inputs = {
    cartero.url = "github:danirod/cartero";
  };
}
```

> [!WARNING]
> This examples assume you're passing `inputs` in the `specialArgs` so you can utilize it
> in others modules if you're splitting your config in multiple files.

Then in your `home.packages` (when using home manager) or
`environment.systemPackages` (global nix packages), add the
derivation.

```nix
environment.systemPackages = [
  inputs.cartero.packages.x86_64-linux.default
];
```

> [!TIP]
> You can try changing the architecture, not tested in every arch atm though.

Another way is by making a nixpkgs overlay to add cartero and then install it
easily.

```nix
nixpkgs.overlays = [
  (_: final: let
    inherit (inputs) cartero;
    inherit (final) system;
  in {
    cartero = cartero.packages.${system}.default
  })
];
```

And then in the packages list of your choice.

```nix
home.packages = with pkgs; [
  cartero
];
```

> [!NOTE]
> You may need to reboot the system or relogin to be able to see cartero on your launcher

## Contributing

> 🐛 This project is currently a larva trying to grow. Do you want to get in?
> Take a seat!

This project is highly appreciative of contributions. If you know about Rust,
GTK or the GNOME technologies and want to help during the development, you can
contribute if you wish. [Fork the project][fork] and commit your code.

Some checklist rules before submitting a pull request:

* **Use a feature branch**, do not make your changes in the trunk branch
  directly.

* **Rebase your code** and make sure that you are working on top of the most
  recent version of the trunk branch, in case something has changed while you
  were working on your code.

* **Update the locales** if you changed strings. The ninja target that you are
  looking for is called `cartero-update-po` (such as `ninja -C build
  cartero-update-po`). Don't worry, you don't have to translate the strings by
  yourself, but make sure that the new templates are added to the .po and .pot
  files.

* **Use the pre-commit hook**. The pre-commit hook will validate that your code
  is formatted. It should be automatically configured if you run Meson in
  development mode (`-Dprofile=development`), but you can install it on your
  own or run `hooks/pre-commit.hook`.

The project is starting small, so if you want to do something big, it is best
to first start a discussion thread with your proposal in order to see how to
make it fit inside the application.

While this application is not official and at the moment is not affiliated with
GNOME, you are expected to follow the [GNOME Code of Conduct][coc] when
interacting with this repository.

## Licenses

Cartero is published under the terms of the GNU General Public License v3.0 or later.

```
Copyright 2024 the Cartero authors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
The Cartero icon is published under the a [Creative Commons
Attribution-ShareAlike 4.0 International license][ccbysa].

## Credits and acknowledgments

Cartero is maintained by [Dani Rodríguez][danirod].

Big shoutout to the [contributors][contrib] who have sent patches or
translations!

Also, Christian suggested Cartero as the name for the application and I liked
it enough to call it like so, therefore shoutout to Christian as well!

Finally, shoutout to many of the GTK and GNOME Circle applications out there whose
source code I've read in order to know how to use some of the GTK features that
you cannot learn just by reading the official docs.

## Blog

Dani's [dev blog][blog] (in Spanish) of Cartero.

[ccbysa]: https://creativecommons.org/licenses/by-sa/4.0/
[coc]: https://conduct.gnome.org
[contrib]: https://github.com/danirod/cartero/graphs/contributors
[danirod]: https://github.com/danirod
[fork]: https://github.com/danirod/cartero/fork
[blog]: https://danirod.es/secciones/devlogs/cartero/
[nightly-flatpak-x86_64]: https://nightly.link/danirod/cartero/workflows/nightly.yaml/trunk/es.danirod.Cartero.Devel-x86_64.zip
[nightly-flatpak-aarch64]: https://nightly.link/danirod/cartero/workflows/nightly.yaml/trunk/es.danirod.Cartero.Devel-aarch64.zip
[nightly-windows-x86_64]: https://nightly.link/danirod/cartero/workflows/nightly.yaml/trunk/es.danirod.Cartero-windows-x86_64.zip
[nightly-macos-sillicon]: https://nightly.link/danirod/cartero/workflows/nightly.yaml/trunk/es.danirod.Cartero-macOS-ARM64.zip
[nightly-macos-intel]: https://nightly.link/danirod/cartero/workflows/nightly.yaml/trunk/es.danirod.Cartero-macOS-X64.zip
