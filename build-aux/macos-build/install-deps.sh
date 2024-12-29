#!/bin/bash

# This script generates a directory called homebrew-arm64 with the dependencies for Cartero
# manually built from source so that they can properly target macOS 11.0. Run this script
# OUTSIDE the main Git repository, because it will conflict with Homebrew install process.
# Set the env after installation before building the app.
#
#   mkdir ~/somewhere-else
#   cd ~/somewhere-else
#   ~/cartero/build-aux/macos-build/install-deps.sh
#   eval "$(~/somewhere-else/homebrew-arm64/bin/brew shellenv)"
#
# To build the dependencies for x86_64, just use arch -x86_64:
#
#   mkdir ~/somewhere-else
#   cd ~/somewhere-else
#   arch -x86_64 ~/cartero/build-aux/macos-build/install-deps.sh
#   eval "$(~/somewhere-else/homebrew-i386/bin/brew shellenv)"
#
# If you are reading this because you came via code search, this script may not work for you,
# but the idea does: Homebrew precompiled bottles are not compatible with older versions of
# macOS, but Homebrew will also ignore the MACOSX_DEPLOYMENT_TARGET environment variable.
# Take a look at this script and see how I am patching some .rb files to avoid ignoring that
# environment variable.
#
# Note that brew install --build-from-source will not build from source the package dependencies.
# This is why I build from source every package in the dependency tree as well, just in case
# any of these packages is also a transitive dependency of mine, so that I can pack a dylib that
# works with the given deployment target.

set -e

export MACOSX_DEPLOYMENT_TARGET=11.0

mkdir homebrew-$(arch) && curl -L https://github.com/Homebrew/brew/tarball/master | tar xz --strip-components 1 -C homebrew-$(arch)
eval "$(homebrew-$(arch)/bin/brew shellenv)"
brew update --force --quiet
export PATH=$PWD/homebrew-$(arch)/bin:$PATH

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
