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
  username = config.system.primaryUser or "root";
  userHome = config.users.users.${username}.home;
  configDir = "${userHome}/.config/palette";
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
          "json"
          "toml"
        ]
      );
      default = { };
      description = "Per-directory format overrides";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ palette ];

    environment.etc."palette/config.toml".source = configFile;

    system.activationScripts.palette = ''
      mkdir -p ${configDir}/palettes ${configDir}/themes
      cp -rn ${palette}/share/palette/palettes/* ${configDir}/palettes/ 2>/dev/null || true
      ln -sf /etc/palette/config.toml ${configDir}/config.toml
    '';
  };
}
