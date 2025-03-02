# Release engineering

This document describes the release process for Cartero; i.e., what to commit when the version is about to get bumped, how to pre-compile the binary packages that are uploaded to the repository.

This process is deliberately not automated in order to assess the quality of the artifacts before uploading them. However, to prevent errors, the process is numbered and it is important to follow the script on every release.

## Preparing for patch releases

Patch releases happen in a branch called release/x.y, where x.y is the major and minor version of Cartero. The patch version is the one that will get bumped (for example, 0.1.2 becomes 0.1.3 and 2.15.1 becomes 2.15.2).

1. Switch to the release branch.
1. Backport every commit and PR of interest into the release branch. To backport a single commit, run `git cherry-pick [commit hash] `. To backport a pull request, pick the merge commit and run `git cherry-pick -m 1 [merge commit hash]`. Assume that sometimes the backport will not be clean and the cherry pick will have conflicts. Release often to prevent this.
1. Update the version number from the following files:
   - Cargo.toml: update the version number in the project metadata.
   - Cargo.lock: run `cargo b` to press the new version number after updating Cargo.toml.
   - build-aux/macos-build.sh: there should be two APP_VERSION variables in the file header.
   - build-aux/macos-installer.sh: there should be two strings declaring the .dmg name.
   - meson.build: there's a version number when declaring the project info.
1. Update the NEWS.md file with the release notes for this version.
1. Reformat the release notes for this version and add them to the releases section of data/cartero.appdata.xml.in.in.
1. Create a release commit, but don't tag it, sign it or push it yet. (If there is an error, it will be easier to correct without force pushing anything or causing double notifications.)

## Collecting artifact files

During the following sections, **artifact** files will be produced. An artifact file is the generated output that gets uploaded to GitHub Releases. It may be used by package managers such as AUR or Homebrew to deliver Cartero to final users. To collect the artifacts:

1. Prepare an empty directory to collect the sources.

These are the artifacts that should be collected:

- The distfile: `cartero-$VER.tar.xz`
- AppImages for each architecture: `Cartero-$VER-$ARCH.AppImage`
- The Windows installer: `cartero-$VER-windows-$ARCH.exe`
- The Windows portable: `cartero-$VER-windows-$ARCH.zip`
- The macOS versrions: `Cartero-$VER-macOS-$ARCH.dmg`

After every artifact is collected, the following should be done:

- SHA-256 checksums of every artifact are collected with `sha256sum * > SHA256SUMS`.
- Every file meant to be released is signed with the GPG key using `for f in $(awk '{ print $2 }' < SHA256SUMS); do gpg --detach-sign --armor $f; done`.

## Create the distfile

This has to happen early after making the release commit. It generates a .tar.xz file with the source code of the version. It also vendors every Rust dependency. The distfile is uploaded to the GitHub release and every binary artifact is also created using this distfile.

This distfile is used to build the application in the Flathub Build Farm, since the farm is not connected to the internet, requiring all the dependencies to be vendored. It is also the safest way to build Cartero because it doesn't require an internet connection, and also guarantees that the dependencies will always be the same.

The distfile is also used by the AUR recipes, for the same reason.

1. Delete `build` and `target` directories, if they exist.
1. Run `meson setup build` followed by `ninja -C build dist`.
1. Assert that the dist tests pass (for instance, the schema, desktop and appstream files are valid, because Flathub fails if the appstream ile is not valid)
1. Check the output of the `build/meson-dist` directory.

**Artifact**: the distfile.

## Linux AppImage

A virtual machine is encouraged to use a clean environment and prevent a polluted development environment. The suggested baseline is either Ubuntu 22.04, Ubuntu 24.04 or Debian 12.

To prevent issues with old dependency versions, Homebrew for Linux is currently used while a better solution appears. Install Linuxbrew (Homebrew for Linux) and install every dependency listed in `MACOS.md`: meson, gtk4, gtksourceview5... You should validate that `brew --prefix` works and that the `lib/` directory is full of shared objects such as `libgtk4` or `libadwaita`.

1. Extract the distfile.
1. `export VENDOR_BASE=$(brew --prefix)`
1. `export LD_LIBRARY_PATH=$(brew --prefix)/lib`
1. `build-aux/appimage-build.sh stable`
1. In a separate terminal try to run `build/appimagedir/Cartero-$ARCH.AppImage`.
1. It is recommended to additionally test on different virtual machines. It should work out of the box in Debian stable without glibc issues.

**Artifact**: the AppImage launcher.

## Windows

The build system in Windows must have MSYS2 and InnoSetup installed. MSYS2 must be installed with the UCRT64 feature. UCRT64 generates a Windows version with no additional DLL dependencies for the crt. Other MSYS versions will require extra dependencies such as libgcc to be packaged with the distribution.

To sign the releases, it is required to install SignTool. This tool is provided by the Windows SDK. Install "Windows SDK Signing Tools for Desktop Apps". You should add it to the PATH, in order to run signtool from PowerShell without having to set the full path in the command manually.

Also, don't use WSL. That would create another GNU/Linux version.

1. Extract the distfile.
1. Run `build-aux/msys-build.sh stable` in the distribution.
1. Check that `build/cartero-win32/bin/cartero.exe` opens and works.
1. Sign the executable:
   - Step 1: `signtool sign /n "[Sign identifier]" /t http://time.certum.pl /fd sha1 /v build/cartero-win32/bin/cartero.exe`
   - Step 2: `signtool sign /n "[Sign identifier]" /tr http://time.certum.pl /fd sha256 /as /v build/cartero-win32/bin/cartero.exe`
1. Bundle the portable version. Switch to the build/cartero-win32 directory and prepare it with `zip -r cartero-$VER-windows-$ARCH.zip bin lib share`.
1. Take the zip out of the build/cartero-win32 directory to prevent adding it to the installer.
1. The build process should have created the file `build/win32-installer.iss`. Compile it with InnoSetup to generate an installer.
1. The installer should be located at `build/Output/cartero.exe`. Test it.
1. Sign the installer:
   - Step 1: `signtool sign /n "[Sign identifier]" /t http://time.certum.pl /fd sha1 /v build/Output/cartero.exe`
   - Step 2: `signtool sign /n "[Sign identifier]" /tr http://time.certum.pl /fd sha256 /as /v build/Output/cartero.exe`
1. Collect the installer (build/Output/cartero.exe) as `cartero-$VER-windows-$arch.exe`.

**Artifacts**: the Windows portable ZIP and the Windows installer.

## macOS

The macOS version will take some time due to the dependency preprocessing. Homebrew is used to provide the dependencies. However, in order to enable support for older systems such as macOS 11, the dependencies are manually recompiled so that the `MACOSX_DEPLOYMENT_TARGET` can be set on every dependency.

Additionally, to avoid polluting the system or the build environment, a separate Homebrew environment should be used. The following script will generate a Homebrew environment.

```bash
#!/bin/bash

set -ex

case "$1" in
arm64)
    DIR=homebrew-arm64
    ;;
i386)
    DIR=homebrew-i386
    ;;
*)
    echo "Usage: $0 [arm64 / i386]"
    exit 1
    ;;
esac

export PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin

export MACOSX_DEPLOYMENT_TARGET=11.0

if ! [ -d $DIR ]; then
git clone git@github.com:Homebrew/brew $DIR
fi

if ! [ -d $DIR/Library/Taps/homebrew ]; then
mkdir -p $DIR/Library/Taps/homebrew
git clone git@github.com:Homebrew/homebrew-core $DIR/Library/Taps/homebrew/homebrew-core
fi

eval "$($DIR/bin/brew shellenv)"
brew update --force --quiet
export PATH=$PWD/$DIR/bin:$PATH

sed -i.bak 's/MACOSX_DEPLOYMENT_TARGET//g' $(brew --prefix)/Library/Homebrew/build_environment.rb
sed -i.bak 's/MACOSX_DEPLOYMENT_TARGET//g' $(brew --prefix)/Library/Homebrew/extend/ENV/shared.rb
rm $(brew --prefix)/Library/Homebrew/build_environment.rb.bak
rm $(brew --prefix)/Library/Homebrew/extend/ENV/shared.rb.bak
```

Invoke it using `./homebrew.sh arm64` to prepare it for Apple Sillicon, or `arch -x86_64 ./homebrew.sh i386` to prepare it for Intel (64 bits).

To download the dependencies, the following script is used:

```bash
#!/bin/bash

set -ex

case "$1" in
arm64)
    DIR=homebrew-arm64
    ;;
i386)
    DIR=homebrew-i386
    ;;
*)
    echo "Usage: $0 [arm64 / i386]"
    exit 1
    ;;
esac

export PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin

export MACOSX_DEPLOYMENT_TARGET=11.0

eval "$($DIR/bin/brew shellenv)"
brew update --force --quiet
export PATH=$PWD/$DIR/bin:$PATH

sed -i.bak 's/MACOSX_DEPLOYMENT_TARGET//g' $(brew --prefix)/Library/Homebrew/build_environment.rb
sed -i.bak 's/MACOSX_DEPLOYMENT_TARGET//g' $(brew --prefix)/Library/Homebrew/extend/ENV/shared.rb
rm $(brew --prefix)/Library/Homebrew/build_environment.rb.bak
rm $(brew --prefix)/Library/Homebrew/extend/ENV/shared.rb.bak

brew install $(brew deps subversion) --build-from-source
brew install subversion --build-from-source
svn list --non-interactive https://svn.code.sf.net/p/netpbm/code/stable

for pkg in meson gtk4 desktop-file-utils pygobject3 adwaita-icon-theme shared-mime-info gtksourceview5 libadwaita; do
brew install $(brew deps $pkg) --build-from-source
brew install $pkg --build-from-source
done
```

Invoke it using `./deps.sh arm64` to prepare it for Apple Silicon, or `arch -x86_64 ./deps.sh i386` to prepare it for Intel (64 bits).

Make sure you run it multiple times until you can confirm that it is not downloading any new dependencies. Sometimes the process fails but continues building anyway.

As a result of running both scripts on both architectures, a directory called `homebrew-i386` should exist with dependencies prepared for the Intel version, and a directory called `homebrew-arm64` should exist with dependencies prepared for the Apple Silicon version.

To build the application, the following steps should be done:

1. Make sure the PATH is reset so that existing Homebrew or MacPorts installations are ignored. For example, `export PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin`.
1. Load the expected Homebrew distribution: `eval "$(homebrew-$(arch)/bin/brew shellenv)"`.
1. Export a variable called `CODESIGN_IDENTITY` with the key ID you get when running `find-identity -p codesigning -v`.
1. Export a variable called `NOTARY_PROFILE` with the notarization profile. If you don't have one, you can create it the following way:
   - Issue an application password on your developer account at <https://account.apple.com/account/manage>.
   - Check the profile for the developer account at <https://developer.apple.com> to get the team ID.
   - Run the following command `xcrun notarytool store-credentials [Profile name] --apple-id [Apple ID] --team-id [Team ID] --password [App Password]`. Then, the notary profile is the value you provided at `[Profile name]`.
1. Extract the distfile and switch to the directory.
1. Run `build-aux/macos-build.sh stable` to build the macOS version of the app.
1. Run `build-aux/macos-build/sign.sh build/cartero-darwin/Cartero.app` to sign the .app file.
1. Run `build-aux/macos-installer.sh stable` to create the installer.
1. Run `for f in build-aux/*.dmg; do xcrun notarytool submit $f --keychain-profile "$NOTARY_PROFILE" --wait; done` to notarize every DMG file.
1. Run `for f in build-aux/*.dmg; do xcrun stapler staple $f; done` to staple the DMG with the notarization result, so that systems do not depend on an internet connection to first run the DMG or the application.

**Artifacts**: the macOS DMG for Apple Silicon and the macOS DMG for Intel 64.
