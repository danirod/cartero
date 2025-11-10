# NixOS Flake

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

<div class="warning">
This examples assume you're passing `inputs` in the `specialArgs` so you can utilize it in others modules if you're splitting your config in multiple files.
</div>

Then in your `home.packages` (when using home manager) or `environment.systemPackages`
(global nix packages), add the derivation.

```nix
environment.systemPackages = [
  inputs.cartero.packages.x86_64-linux.default
];
```

> **Tip**: You can try changing the architecture, not tested in every arch atm though.

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

> **Note**: You may need to reboot the system or relogin to be able to see cartero on your launcher
