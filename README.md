# palette

A terminal colour palette manager. Browse, edit, and manipulate colour palettes stored as plain JSON files. Designed to be the single source of truth for your system's colour scheme.
Every app reads from the same palette, or set each app to have its own palette. Either way, `palette` makes that easier by centralizing as much of the colour palette process into a tui.

Built with [ratatui](https://ratatui.rs/) and [pastel](https://github.com/sharkdp/pastel) for perceptual colour math.

---

## Features

- Browse palettes in a two pane TUI. list on the left, live preview on the right
- Edit colours with keyboard controls: RGB channels, hue rotation, saturation, lightness
- Find the 3 closest CSS/X11 named colours and 3 closest palette entries using CIEDE2000 perceptual distance
- Preview colour pairs for contrast checking (text-on-background / background-on-text)
- Generate random colours and save them as new palette entries
- Copy hex, RGB, or HSL to clipboard
- Create new palettes and add directories from within the TUI (directories must exist)
- Auto-creates a default theme on first run
- Supports map, array-of-objects, and array-of-tuples JSON formats

---

## Installation

### Nix (flake)

Add palette to your flake inputs:
> [!Note]
> `palette` follows nixpkgs 26.05 internally

```nix
{
  inputs = {
    palette-tui.url = "github:Khitboksy/palette-tui";
    // palette-tui.inputs.nixpkgs.follows = "nixpkgs";
  };
}
```

Then enable the module in your home-manager or NixOS configuration:

**Modules** (in `home|configuration.nix`):

```nix
# This is for Home-manager, as thats what i use.
{ inputs, ... }:
{
  imports = [ 
   inputs.palette-tui.homeManagerModules.default
   # Replace the home-manager module with this comment for the nixos module
   # inputs.palette-tui.nixosModules.default
  ];

  # Options are the same for the nixos module, just a different import
  programs.palette-tui = {
    enable = true;
 # All options are optional.  # these are the defaults:
    # defaultDir = null;         # default palette directory to open  
    # themeFile = "theme.json";  # theme palette name or absolute path
    # extraDirs = [];            # additional directories to scan
    # dirFormats = {};           # per-directory save format overrides
  };
}
```

Then rebuild

### Non-Nix (Arch, Ubuntu, Fedora, etc.)

Run the [install script](./install.sh):

```bash
curl -fsSL https://raw.githubusercontent.com/Khitboksy/palette-tui/main/install.sh | bash
```

This downloads a pre-built binary for your platform, installs it to `/usr/local/bin/`, and copies [sample palettes](./palettes) to `~/.config/palette/palettes/`.

For atomic distros (Bazzite, Silverblue), install to user-local instead:

```bash
BIN_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/Khitboksy/palette-tui/main/install.sh | bash
```

[Uninstall](./uninstall.sh):

```bash
rm /usr/local/bin/palette && rm -rf ~/.config/palette
```

### Nix (non-flake, channels)

The package expression is at [`nix/package.nix`](./nix/package.nix). Use it in your NixOS or home-manager config:

```nix
{ pkgs, ... }:
let 
  palette = pkgs.callPackage /path/to/package.nix {};
in
{
  environment.systemPackages = [
    palette
  ];
}
```

---

## Quickstart

After installing, run:

```bash
palette
```

On first launch, palette creates `~/.config/palette/themes/theme.json` with default colours. You will see the theme palette loaded in the left pane with the Catppuccin Mocha sample palette available.

**First steps:**

1. Press `Tab` to open the palette selector. You will see your palettes grouped by directory.
2. Press `j`/`k` to navigate, `Enter` to switch palettes. Try the Catppuccin Mocha palette.
3. Press `j`/`k` to select a colour. The right pane shows a live preview with a swatch, and some text examples.
4. Press `Enter` to enter Command mode. Press `s` to copy the hex to clipboard, or `c` to copy the rgb to clipboard.
5. Press `e` to enter Edit mode. Use `r`/`R` (red), `g`/`G` (green), `b`/`B` (blue) to adjust channels. Use `j`/`J` for hue, `l`/`L` for lightness, `k`/`K` for saturation.
6. Press `w` to save. Type a name and press `Enter`.
7. Press `z` to return to Preview mode. Press `q` to quit.

---

## Keyboard reference

### Preview mode (default)

| Key | Action |
|-----|--------|
| `j`/`k` or `Up`/`Down` | Move selection |
| `Enter` | Enter Command mode |
| `h`/`l` or `Left`/`Right` | Previous/next palette |
| `Tab` | Open palette selector |
| `r` | Generate random colour |
| `e` | Edit current colour |
| `.` | Reload theme |
| `q`/`Esc` | Quit |

### Command mode

| Key | Action |
|-----|--------|
| `z`/`Esc` | Back to Preview |
| `e` | Edit current colour |
| `i` | Input hex directly |
| `s`/`c`/`d` | Copy hex/rgb/hsl to clipboard |
| `f` | Flip to complementary colour |
| `1`-`3` | Jump to similar named colour |
| `4`-`6` | Jump to similar palette entry |
| `p` | Pick a pair for contrast preview |
| `P` | Clear pair |
| `Tab` | Open palette selector |

### Edit mode

| Key | Action |
|-----|--------|
| `w` | Save (overwrite or new entry) |
| `e`/`Esc` | Back to Command |
| `z` | Back to Preview |
| `c` | Clear colour (empty hex) |
| `r`/`R` | Red +1/-1 |
| `g`/`G` | Green +1/-1 |
| `b`/`B` | Blue +1/-1 |
| `j`/`J` | Hue +1/-1 degree |
| `l`/`L` | Lighten/darken 1% |
| `k`/`K` | Saturate/desaturate 1% |
| `p`/`P` | Pick/clear pair |

### Palette Selection Mode

| key | action |
| --- | --- |
| `j`/`k` or `Up`/`Down` | Move slection |
| `a` | Add a new palette |
| `n` | Add a new directories (adds to config.toml) |
| `f` | Filter output formats for a given directory |
| `Enter` | Select palette |

---

## Configuration

Config lives at `~/.config/palette/config.toml`. Auto-created on first run.

```toml
# Default palette directory (falls back to ~/.config/palette/palettes)
default_dir = "/home/you/my-palettes"

# Theme palette (
 # filename: local to ~/.config/palette/theme
 # absolute path: anywhere on your machine
#)
theme_palette = "theme.json"

# Extra directories to scan for palettes
# Note: Directories must exist.
extra_dirs = ["/home/you/shared-palettes", "/opt/corporate-palettes"]

# Per-directory save format: which fields to write (hex, hsl, rgb)
# Omit a directory to save all fields.
[dir_formats]
"/home/you/my-palettes" = ["hex"]
```

---

## JSON format

Palette files are plain JSON. Three formats are accepted:

**Map format** (canonical):

```json
{
  "rosewater": {
    "hex": "#f5e0dc",
    "hsl": "hsl(10, 76%, 91%)",
    "rgb": "rgb(245, 224, 220)"
  },
  "mauve": {
    "hex": "#cba6f7",
    "hsl": "hsl(267, 84%, 88%)",
    "rgb": "rgb(203, 166, 247)"
  }
}
```

**Array of objects:**

```json
[
  { "name": "rosewater", "hex": "#f5e0dc" },
  { "name": "mauve", "hex": "#cba6f7" }
]
```

**Array of tuples:**

```json
[
  ["rosewater", "#f5e0dc"],
  ["mauve", "#cba6f7"]
]
```

All formats are normalised to the map format on save. Missing `hsl`/`rgb` fields are recomputed from hex.

> [!Warning]  
> Colour names are sanitized to [a-zA-Z0-9_-] on load. Names with other characters will be altered
> (e.g., `my colour` becomes `my-colour`). This ensures palette files remain scriptable with jq and other tools.

---

## Use cases

palette outputs a JSON object with hex, HSL, and RGB values for each named colour. This makes it a universal colour source. **Any** app that accepts hex, rgb, or hsl can read from it. Here are some ways to integrate palette into your workflow.

### Limitations

 Most applications will require you to re-run their respective palette-to-config script when you update your colours. Setting up a file watcher to automate this is beyond the scope of this readme. These examples are here to give you a gauge of how this program can be utilized, not a tutorial on how to take full advantage of the tool.
I personally dont even know what the tools limits, strengths, or weaknesses are!

### NixOS: custom colour library

If you use NixOS or home-manager, you can read palette's JSON files directly in your config and use them as a colour library. This means you define your colours once in palette, and every NixOS module references them:

1. Create a `colors.nix`

This step is technically optional, but id recommend it so you can import this globally. That way if you need colours in multiple files youre not having to call a bunch of imports, or having to repeat this code multiple times across your codebase

```
#./colors.nix
builtins.fromJSON (
    builtins.readFile (
      builtins.toPath "~/.config/palette/palettes/catppuccin-mocha.json"
    )
  );
```

2. Wire it into your config

```nix
{ pkgs, ... }:

let
  # Import the colours
  colors = import ./colors.nix;
  # Abstract for clarity
  bg = colors.base.hex;
  fg = colors.text.hex;
  accent = colors.blue.hex;
in

{
  programs.kitty.settings = {
    background = bg;
    foreground = fg;
    color0 = colors.surface0.hex;
    color1 = colors.red.hex;
    color4 = colors.blue.hex;
  };

  programs.waybar.style = ''
    * { background: ${bg}; color: ${fg}; }
    #workspaces button.active { background: ${accent}; }
  '';

  programs.alacritty.settings.colors.primary = {
    background = bg;
    foreground = fg;
  };
}
```

One palette, every app themed. No scripts, no code generation, just Nix expressions reading the JSON.

> [!Important]
> This integration *does* work with a set-and-forget mindset. Update a colour in `palette` and rebuild.
> You should see the colours update

### Niri: compositor theming

Niri reads colours from `~/.config/niri/config.kdl`. You can generate a KDL snippet from your palette and include it.

Create `~/.config/niri/colors.kdl`:

```bash
#!/usr/bin/env bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
jq -r '
  "layout {",
  "  focus-ring {",
  "    active-color \"" + .blue.hex + "\"",
  "    inactive-color \"" + .surface0.hex + "\"",
  "    urgent-color \"" + .red.hex + "\"",
  "  }",
  "  border {",
  "    active-color \"" + .blue.hex + "\"",
  "    inactive-color \"" + .surface0.hex + "\"",
  "    urgent-color \"" + .red.hex + "\"",
  "  }",
  "  shadow { color \"" + .crust.hex + "\" }",
  "}"
' "$PALETTE" > ~/.config/niri/colors.kdl
```

Then in `~/.config/niri/config.kdl`:

```kdl
include "~/.config/niri/colors.kdl"
```

Niri live-reloads on save.

> This does not handle live edits. See [Scripting pipeline](#scripting-pipeline) for a file watcher example.

### Display managers

**SDDM**: edit `theme.conf.user` in your SDDM theme directory:

```bash
#!/usr/bin/env bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
THEME_DIR="/usr/share/sddm/themes/your-theme"
BG=$(jq -r '.base.hex' "$PALETTE")
ACCENT=$(jq -r '.blue.hex' "$PALETTE")

sudo tee "$THEME_DIR/theme.conf.user" <<EOF
[General]
Background=$BG
TopBorder=$ACCENT
EOF
```

**LightDM (GTK greeter)** -- accepts hex directly:

```ini
[greeter]
background = #1e1e2e
```

Generate it:

```bash
jq -r '.base.hex' "$HOME/.config/palette/palettes/catppuccin-mocha.json" | \
  xargs -I{} sudo sed -i "s|^background=.*|background={}|" /etc/lightdm/lightdm-gtk-greeter.conf
```

> Display managers require a re-login to apply changes. See [Limitations](#limitations) for details.

### Noctalia

Noctalia uses Material Design 3 roles (`mPrimary`, `mSurface`, etc.), not flat named colours. Map your palette to its format:

```bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
OUT="$HOME/.config/noctalia/palettes/catppuccin-mocha.json"

jq '{
  dark: {
    mPrimary: .blue.hex,
    mSurface: .surface0.hex,
    mOnSurface: .text.hex,
    mError: .red.hex
  },
  terminal: {
    background: .base.hex,
    foreground: .text.hex
  }
}' "$PALETTE" > "$OUT"
```

Then select it in `settings.toml`:

```toml
[theme]
source = "custom"
custom_palette = "catppuccin-mocha"
```

> See [Limitations](#limitations) for details.

### Terminal emulators

> See [Limitations](#limitations) for details

**kitty**: generate a colours file:

```bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
OUT="$HOME/.config/kitty/colors.conf"

jq -r '
  "background " + .base.hex,
  "foreground " + .text.hex,
  "color0 " + .surface0.hex,
  "color1 " + .red.hex,
  "color2 " + .green.hex,
  "color3 " + .yellow.hex,
  "color4 " + .blue.hex,
  "color5 " + .mauve.hex,
  "color6 " + .teal.hex,
  "color7 " + .subtext1.hex
' "$PALETTE" > "$OUT"
```

Then in `~/.config/kitty/kitty.conf`:

```
include ~/.config/kitty/colors.conf
```

**alacritty**: generate an importable theme file:

```bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
OUT="$HOME/.config/alacritty/themes/palette.toml"

jq -r '
  "[colors.primary]",
  "background = \"" + .base.hex + "\"",
  "foreground = \"" + .text.hex + "\"",
  "",
  "[colors.normal]",
  "black = \"" + .surface0.hex + "\"",
  "red = \"" + .red.hex + "\"",
  "green = \"" + .green.hex + "\"",
  "blue = \"" + .blue.hex + "\""
' "$PALETTE" > "$OUT"
```

Then in `~/.config/alacritty/alacritty.toml`:

```toml
import = ["~/.config/alacritty/themes/palette.toml"]
```

**foot**:
> note: foot uses hex without the `#`:

```bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
OUT="$HOME/.config/foot/foot.ini"

jq -r '
  "[colors]",
  "background=" + (.base.hex | ltrimstr("#")),
  "foreground=" + (.text.hex | ltrimstr("#"))
' "$PALETTE" > "$OUT"
```

### Waybar

Generate a colours file from your palette:

> Unlike the others, this example utilizes CSS, meaning we get to use `rgb` values!
> See [Limitations](#limitations) for more details.

```bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
OUT="$HOME/.config/waybar/colors.css"

jq -r '
  "@define-color bg " + .base.rgb + ";",
  "@define-color fg " + .text.rgb + ";",
  "@define-color accent " + .blue.rgb + ";"
' "$PALETTE" > "$OUT"
```

Then in `style.css`:

```css
@import url("colors.css");
* { background: @bg; color: @fg; }
#workspaces button.active { background: @accent; }
```

Reload: `killall -SIGUSR2 waybar`

### GTK

> Much like waybar, GTK uses CSS, meaning we can utilize `hsl` values!
> See [Limitations](#limitations) for details.

Generate from palette:

```bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
OUT="$HOME/.config/gtk-3.0/gtk.css"

jq -r '
  "@define-color accent_color " + .blue.hsl + ";",
  "@define-color bg_color " + .base.hsl + ";",
  "@define-color fg_color " + .text.hsl + ";"
' "$PALETTE" > "$OUT"
```

## Scripting pipeline

### Niri

The one-shot script [above](#niri-compositor-theming) works, but you can automate it with `inotifywait` from `inotify-tools`. This watches your palette file and regenerates the KDL snippet on every save.

```bash
#!/usr/bin/env bash
PALETTE="$HOME/.config/palette/palettes/catppuccin-mocha.json"
OUT="$HOME/.config/niri/colors.kdl"

apply() {
  jq -r '
    "layout {",
    "  focus-ring {",
    "    active-color \"" + .blue.hex + "\"",
    "    inactive-color \"" + .surface0.hex + "\"",
    "    urgent-color \"" + .red.hex + "\"",
    "  }",
    "  border {",
    "    active-color \"" + .blue.hex + "\"",
    "    inactive-color \"" + .surface0.hex + "\"",
    "    urgent-color \"" + .red.hex + "\"",
    "  }",
    "  shadow { color \"" + .crust.hex + "\" }",
    "}"
  ' "$PALETTE" > "$OUT"
}

apply  # generate on start
inotifywait -m -e modify "$PALETTE" | while read -r; do apply; done
```

Run this in the background at login (e.g. a systemd user service or your shell rc). Niri live-reloads the `include`'ed file automatically.

---

## Building from source

Requires Rust 1.75+.

```bash
git clone https://github.com/Khitboksy/palette-tui
cd palette-tui
cargo build --release
./target/release/palette
```

For development:

```bash
cargo watch -x check    # auto-check on save
cargo watch -x run      # auto-run on save
cargo clippy            # lint
cargo fmt               # format
```

---

## Dependencies

- [ratatui](https://ratatui.rs/) -- terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) -- terminal manipulation
- [pastel](https://github.com/sharkdp/pastel) -- colour science (CIEDE2000, conversions)
- [serde](https://serde.rs/) + [serde_json](https://github.com/serde-rs/json) -- JSON serialisation
- [arboard](https://github.com/1Password/arboard) -- clipboard access
- [toml](https://github.com/toml-rs/toml) -- config parsing
- [rand](https://rust-random.github.io/book/) -- random colour generation
