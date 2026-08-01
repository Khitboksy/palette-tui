{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShellNoCC {
  packages = with pkgs; [
    rustc
    cargo
    clippy
    rustfmt
    cargo-watch
    gcc
  ];
}
