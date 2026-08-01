{
  description = "Terminal colour palette manager";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "palette";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            postInstall = ''
              mkdir -p $out/share/palette/palettes
              cp palettes/*.json $out/share/palette/palettes/
            '';
          };
        }
      );

      devShells = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShellNoCC {
            packages = with pkgs; [
              rustc
              cargo
              clippy
              rustfmt
              cargo-watch
              gcc
            ];
          };
        }
      );

      homeManagerModules.default = import ./modules/hm.nix { inherit self; };

      nixosModules.default = import ./modules/nixos.nix { inherit self; };

    };
}
