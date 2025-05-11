{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [
    cargo
    pkg-config
  ];
  buildInputs = [
    udev alsa-lib-with-plugins vulkan-loader
    libxkbcommon wayland # To use the wayland feature
  ] ++ (with xorg; [
    libX11 libXcursor libXi libXrandr # To use the x11 feature
  ]);
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  packages = [
    rust-analyzer
    rustfmt
    clippy
  ];
}
