{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShellNoCC {
  packages = with pkgs; [
    # Rust
    rustc
    cargo
    clippy
    rustfmt
    rust-analyzer
    cargo-watch
    gcc

    # Nix
    nixfmt-tree
    nixd
    statix
    deadnix
  ];
  env = {
    DEV_OPTIONS = "1";
  };
  shellHook = ''
    echo "palette-tui dev environment"
    echo "  rust fmt:   cargo fmt"
    echo "  rust lsp:   rust-analyzer"
    echo "  nix fmt:    nixfmt-tree"
    echo "  nix lsp:    nixd"
    echo "  nix lint:   statix. deadnix"
  '';
}
