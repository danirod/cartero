<p align="center"><img src="data/icons/scalable/apps/es.danirod.Cartero.svg" width="256" height="256" alt=""></p>

<h1 align="center">Cartero</h1>
<p align="center">Make HTTP requests and test APIs</p>

<p align="center">
<img src="data/screenshots/cartero-default.png" alt="Screenshot of Cartero">
</p>

Cartero is a graphical HTTP client that can be used as a developer tool to
test web APIs and perform all kind of HTTP requests to web servers. It is
compatible with any REST, SOAP or XML-RPC API and it supports multiple request
methods as well as attaching body payloads to compatible requests.

**Features:**

- Loads and saves to plain Git-friendly TOML files, so that you can own your data.
- Customization and modification of the request headers and body payloads.
- Variable binding for API keys and other secret information.

**Motivation:**

This project exists because there aren't many native graphical HTTP testing
applications / graphical alternatives to cURL that are fully free software, and
I think the world has had enough of Electron / non-native applications that are
anonymously accesible until one day you are forced to create an account and
log in to use just to make some investor happy with their numbers or to chug
some unwanted artificial intelligence at users.

## Download

### Get it from Flathub

<a href="https://flathub.org/apps/es.danirod.Cartero">
<img width="240" alt="Get it on Flathub" src="https://flathub.org/api/badge?svg&locale=en">
</a>

### Download for your OS

| Windows | macOS | AppImage |
| ------- | ----- | --- |
| ![Windows](doc/windows.png) | ![macOS](doc/macos.png) | ![AppImage](doc/appimage.png) |
| [x64, installer][windows-x86_64]<br>[x64, portable][windows-portable-x86_64] | [Apple Sillicon][macos-sillicon]<br>[Intel 64-bit][macos-intel] | [amd64][appimage-x86_64] |

### Get it from your package manager

Note: distributions in package managers are maintained by the community. While I am open to provide help and communication with maintainers of those ports, outdated versions and other packaging issues should be reported first to the package manager or to the package maintainer, not upstream.

[![Packaging status](https://repology.org/badge/vertical-allrepos/cartero.svg)](https://repology.org/project/cartero/versions)

Additional platforms:

<details>
  <summary><strong>macOS: install via Homebrew</strong></summary>

To install Cartero from this cask, simply add this repository as a tap.

```
brew tap SoloAntonio/cartero
```

Now you can install any version hosted as cask with

```
brew install --cask cartero
```
</details>

<details>
  <summary><strong>NixOS: manually add the flake</strong></summary>

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

Then in your `home.packages` (when using home manager) or `environment.systemPackages`
(global nix packages), add the derivation.

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

</details>

## Building

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
meson setup build --prefix=/usr
ninja -C build
ninja -C build install
```

## Hacking and contributing

**If you plan on contributing to the project**, use the development profile.
It will also configure a Git hook so that the source code is checked prior to
authoring a Git commit. The hook runs `cargo fmt` to assert that the code is
formatted. Read `hooks/pre-commit.hook` to inspect what the script does.

```sh
meson setup build -Dprofile=development
```

If you want to hack the source code and make your own changes to Cartero, you
can do it as long as you know enough Rust and enough about GTK and the rest of the
libraries it uses. Check out the [HACKING.md](HACKING.md) file. It provides instructions
useful for those who want to compile, test and run the application, specifically how to
compile the resource bundles and run the application.

If you want to share your changes with the world, you could send a pull request to
add the code to Cartero so that anyone can benefit from it. Information on how to
contribute has moved to [CONTRIBUTING.md](CONTRIBUTING.md).

**Other ways to contribute to Cartero also include reporting bugs, sending feedback,
talking about Cartero to other people to make the project more popular, and sending
translations**. We are using [Weblate][weblate] to coordinate and translate comfortably
this project using a web interface. Make an account and start proposing strings and they
will be added to the application. That will also entitle you as a contributor!

## Licenses

Cartero is published under the terms of the GNU General Public License v3.0 or later.

```
Copyright 2024-2025 the Cartero authors

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

Cartero is maintained by Dani Rodríguez.

Big shoutout to the [contributors][contrib] who have sent patches or
translations! Also, Christian suggested Cartero as the name for the
application and I liked it enough to call it like so, therefore shoutout
to Christian as well!

[ccbysa]: https://creativecommons.org/licenses/by-sa/4.0/
[contrib]: https://github.com/danirod/cartero/graphs/contributors
[weblate]: https://hosted.weblate.org/projects/cartero/
[windows-x86_64]: https://github.com/danirod/cartero/releases/download/v0.1.4/Cartero-0.1.4-windows-x64.exe
[windows-portable-x86_64]: https://github.com/danirod/cartero/releases/download/v0.1.4/Cartero-0.1.4-windows-x64.zip
[macos-sillicon]: https://github.com/danirod/cartero/releases/download/v0.1.4/Cartero-0.1.4-macOS-arm64.dmg
[macos-intel]: https://github.com/danirod/cartero/releases/download/v0.1.4/Cartero-0.1.4-macOS-x64.dmg
[appimage-x86_64]: https://github.com/danirod/cartero/releases/download/v0.1.4/Cartero-0.1.4-x86_64.AppImage
