# Installing Cartero

## Using Flatpak

The easiest way to get Cartero on your system if you use GNU/Linux is to use the **Flatpak**.
The Flatpak version of Cartero [available on Flathub][flathub] is official.

Make sure that Flatpak is installed on your system, or install Flatpak if you haven't yet.
There are instructions on how to install Flatpak and configure the Flathub repository in
flathub.org. [Pick your distro][instructions] and do the process.

If you are using GNOME or KDE as a desktop environment, you should be able to locate Cartero
in GNOME Software or in KDE Discover. Otherwise, you can always install Cartero from the command
line by issuing the following command:

```bash
flatpak install es.danirod.Cartero
```

Accept the changes and Cartero will be installed.

You will be able to start Cartero using the application launcher or application menu that your
desktop environment is using. Normally this involves either searching for Cartero from the icon
grid, or locating Cartero in the corresponding application menu. In any case, you should be able
to start Cartero from the command line as well by running the following command:

```bash
flatpak run es.danirod.Cartero
```

## Alternative methods

### AppImage

There is support for AppImage. You should be able to grab an official AppImage bundle and execute it.
[There are links in the main website][cartero]. Depending on your distro and desktop environment,
you should be able to start the application from the command line or by double clicking it from your
file manager.

### Build from source

You can also download a tarball with the source code of Cartero. Locate the latest version available
from the [releases section of the GitHub repository][releases], and get the .tar.xz file. This tarball
contains the source code of Cartero, as well as every vendored Rust dependency.

Alternatively, you can clone the whole repository and pick which version you want to compile or use
the latest tip from the Git repository, although this might cause unexpected issues if the tip is
buggy.

```bash
git clone https://github.com/danirod/cartero ~/cartero
```

Follow the [build instructions](./compiling.md) to compile and install Cartero on your system.

### Use other package managers

Cartero may be available in other package managers. Many of these package managers are tracked in
[Repology][repology].

Note that these packages may be provided by other maintainers not related to Cartero. Any issues,
specifically regarding missing or outdated packages, should be reported to the package manager
maintainer, not to the Cartero project.

The maintainers of Cartero will be happy to provide help in any way if there are issues or questions
during the repackaging of Cartero for a specific platform.

[flathub]: https://flathub.org/apps/es.danirod.Cartero
[instructions]: https://flathub.org/setup
[cartero]: https://cartero.danirod.es
[releases]: https://github.com/danirod/cartero/releases
[repology]: https://repology.org/project/cartero/versions
