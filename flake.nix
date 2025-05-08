{
  description = "Rust Wallpaper GUI for Hyprland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable."1.81.0".default;
      in {
        packages.default = pkgs.buildRustPackage {
          pname = "wallpaper-manager";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            rust
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
          ];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust
            pkg-config
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
          ];
        };
      });
}
