{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "cartero";
  version = "git";

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = with pkgs; [
    meson
    ninja
    cargo
    rustc
    pkg-config
    blueprint-compiler
    desktop-file-utils
    gtk4
    shared-mime-info
    glib
    wrapGAppsHook
    hicolor-icon-theme
  ];

  buildInputs = with pkgs; [
    gtksourceview5
    pango
    gdk-pixbuf
    openssl_3_3
    graphene
    libadwaita
  ];

  desktopItems = with pkgs; [
    (makeDesktopItem rec {
      desktopName = "Cartero";
      name = lib.toLower desktopName;
      comment = "Make HTTP requests and test APIs.";
      exec = "cartero";
      tryExec = exec;
      icon = "es.danirod.Cartero.svg";
      keywords = [ "Gnome" "GTK" "HTTP" "RESET" ];
      categories = [ "GNOME" "GTK" "Network" "Development" ];
      terminal = false;
      type = "Application";
    })
  ];

  configurePhase = ''
    runHook cargoSetupHook
    runHook mesonConfigurePhase
  '';

  meta = with pkgs.lib; {
    description = "Make HTTP requests and test APIs";
    license = licenses.gpl3Only;
    mainProgram = "cartero";
    maintainers = with maintainers; [
      danirod
      alphatechnolog
    ];
  };
}
