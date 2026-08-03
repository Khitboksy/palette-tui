{
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "palette";
  version = "1.0.1";
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;
  postInstall = ''
    mkdir -p $out/share/palette/palettes
    cp palettes/*.json $out/share/palette/palettes/
  '';
}
