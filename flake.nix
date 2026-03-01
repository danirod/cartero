{
  description = "Make HTTP requests and test APIs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          rust-overlay.overlays.default
          (_: final: {
            cartero = final.callPackage ./default.nix {
              pkgs = final;
              rustc = final.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
              cargo = final.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
            };
          })
        ];

        pkgs = import nixpkgs {
          inherit
            system
            overlays
            ;
        };
      in {
        packages = rec {
          inherit (pkgs) cartero;
          default = cartero;
        };

        devShells = with pkgs; let
          buildShellRecipe = {
            LD_LIBRARY_PATH = lib.makeLibraryPath [
              gtk4
              libadwaita
              glib
            ];

            nativeBuildInputs = [
              meson
              ninja
              python313
              cargo
              rustc
              rustfmt
              pkg-config
              blueprint-compiler
              desktop-file-utils
              gtk4
              shared-mime-info
              glib
              wrapGAppsHook4
              hicolor-icon-theme
            ];

            buildInputs = [
              gtksourceview5
              pango
              gdk-pixbuf
              openssl
              graphene
              libadwaita
            ];
          };

          launchShellRecipe = {
            buildInputs = [cartero];
          };
        in rec {
          # use by default the full dev env.
          default = mixedShell;

          # a dev shell which helps to only build cartero manually without installing the dependencies.
          buildingShell = mkShell (buildShellRecipe
            // {
              name = "building-shell";

              shellHook = ''
                echo "> In this shell you should be able to build cartero manually without installing"
                echo "> the dependencies since they already come installed automatically."
              '';
            });

          # useful to test cartero by using nix develop --command cartero
          launchShell = mkShell (launchShellRecipe
            // {
              name = "launch-shell";

              shellHook = ''
                echo "> Here you can launch cartero by typing \`cartero\` in this interactive shell"
                echo "> or by using nix develop --command cartero instead"
              '';
            });

          # both
          mixedShell = mkShell {
            name = "full-dev-env";

            inherit
              (buildShellRecipe)
              LD_LIBRARY_PATH
              ;

            buildInputs =
              []
              ++ buildShellRecipe.buildInputs
              ++ launchShellRecipe.buildInputs;

            shellHook = ''
              echo "> In this shell you should be able to either build cartero manually without installing"
              echo "> the dependencies manually, or by running \`cartero\` you could launch a prebuilt version instead"
            '';
          };
        };
      }
    );
}
