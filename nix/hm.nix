{ self }:
{
  pkgs,
  lib,
  config,
  ...
}:

let
  cfg = config.programs.palette-tui;
  system = pkgs.stdenv.hostPlatform.system;
  palette = self.packages.${system}.default;
  tomlFormat = pkgs.formats.toml { };
  configFile = tomlFormat.generate "palette-config.toml" {
    default_dir = cfg.defaultDir;
    theme_palette = cfg.themeFile;
    extra_dirs = cfg.extraDirs;
    dir_formats = cfg.dirFormats;
  };
in
{
  options.programs.palette-tui = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Whether to enable palette-tui";
    };

    defaultDir = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Default palette directory to open on launch";
    };

    themeFile = lib.mkOption {
      type = lib.types.str;
      default = "theme.json";
      description = ''
        Theme palette file name (relative to ~/.config/palette/themes/)
        or an absolute path.
      '';
    };

    extraDirs = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Additional directories to scan for palettes";
    };

    dirFormats = lib.mkOption {
      type = lib.types.attrsOf (
        lib.types.enum [
          "hex"
          "rgb"
          "hsl"
        ]
      );
      default = { };
      description = "Per-directory format overrides";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ palette ];

    xdg.configFile = {
      "palette/config.toml".source = configFile;
      "palette/palettes".source = "${palette}/share/palette/palettes";
    };
  };
}
