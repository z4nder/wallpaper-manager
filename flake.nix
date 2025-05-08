{
  description = "Wallpaper Manager (Hyprland) - Local crate flake";

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
        rust = pkgs.rust-bin.stable."1.86.0".default;
      in {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "wallpaper-manager";
          version = "0.1.0";
          src = ./.; # 👈 usa o código local

          nativeBuildInputs = [ rust pkgs.pkg-config pkgs.makeWrapper ];
          buildInputs = with pkgs; [
            libxkbcommon
            wayland
            wayland-protocols
            xorg.libX11
            xorg.libXcursor
            libGL
            vulkan-loader
          ];

          buildPhase = ''
            export CARGO_HOME=$(mktemp -d)
            cargo build --release
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/wallpaper-manager $out/bin/
            wrapProgram $out/bin/wallpaper-manager \
              --set LD_LIBRARY_PATH "${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:${pkgs.xorg.libX11}/lib:${pkgs.libGL}/lib:${pkgs.vulkan-loader}/lib:$LD_LIBRARY_PATH"
          '';
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          name = "wallpaper-manager";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust
            pkg-config
            libxkbcommon
            wayland
            xorg.libX11
            libGL
            vulkan-loader
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:${pkgs.xorg.libX11}/lib:${pkgs.libGL}/lib:${pkgs.vulkan-loader}/lib:$LD_LIBRARY_PATH"
            echo "🦀 Ambiente pronto: use 'cargo run -- gui'"
          '';
        };
      });
}
