{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config

    wayland
    libxkbcommon
    xorg.libX11
    libGL
    vulkan-loader
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:${pkgs.xorg.libX11}/lib:${pkgs.libGL}/lib:${pkgs.vulkan-loader}/lib:$LD_LIBRARY_PATH"
    echo "🔧 Ambiente pronto. Execute: cargo run -- gui"
  '';
}
