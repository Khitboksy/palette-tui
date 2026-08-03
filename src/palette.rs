use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app;
use crate::colour::{color_to_hex, format_hsl, format_rgb, hex_to_color};
use crate::helpers;
use pastel::Color;

// XDG helpers
fn xdg_config_home() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(dir)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        PathBuf::from(".")
    }
}

fn config_path() -> PathBuf {
    xdg_config_home().join("palette").join("config.toml")
}

pub fn default_palettes() -> PathBuf {
    xdg_config_home().join("palette").join("palettes")
}

pub fn themes_dir() -> PathBuf {
    xdg_config_home().join("palette").join("themes")
}

// Config
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub default_dir: Option<String>,
    #[serde(default)]
    pub extra_dirs: Vec<String>,
    #[serde(default)]
    pub dir_formats: HashMap<String, Vec<String>>,
    // Theme palette filename (e.g. "theme.json"). Lives in themes_dir().
    #[serde(default = "default_theme_palette")]
    pub theme_palette: String,
}

fn default_theme_palette() -> String {
    "theme.json".to_string()
}

impl Config {
    // Load config from $XDG_CONFIG_HOME/palette/config.toml.
    // Creates a default config file if none exists.
    pub fn load() -> Self {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(data) => toml::from_str(&data).unwrap_or_default(),
            Err(_) => {
                // Create default config file
                let default_content = "# palette configuration\n\
                    # default_dir = \"/path/to/palettes\"\n\
                    extra_dirs = []\n\
                    \n\
                    # Per-directory save formats\n\
                    # Only the listed fields will be written to JSON.\n\
                    # If a directory is not listed, all fields are saved.\n\
                    # Valid fields: \"hex\", \"hsl\", \"rgb\"\n\
                    # [dir_formats]\n\
                    # \"/home/user/my-theme-dir\" = [\"hex\"]\n";
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&path, default_content);
                Self::default()
            }
        }
    }

    // Save config to $XDG_CONFIG_HOME/palette/config.toml.
    // Creates parent directories if needed.
    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let data = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, data).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    // Get the default palette directory path.
    // Uses config value if set, otherwise $XDG_CONFIG_HOME/palette/palettes.
    pub fn default_dir_path(&self) -> PathBuf {
        match &self.default_dir {
            Some(d) => PathBuf::from(d),
            None => default_palettes(),
        }
    }

    // Get all directories in scan order:
    //   1. themes dir (always)
    //   2. configured default dir (if set)
    //   3. internal palettes dir (always -- hidden when empty)
    //   4. extra dirs from config
    pub fn all_dirs(&self) -> Vec<PathBuf> {
        let themes = themes_dir();
        let internal_palettes = default_palettes();
        let default = self.default_dir_path();
        let mut extras: Vec<PathBuf> = self.extra_dirs.iter().map(PathBuf::from).collect();
        extras.sort();
        let mut dirs = Vec::new();
        // 1. Themes (always)
        if !dirs.contains(&themes) {
            dirs.push(themes);
        }
        // 2. Configured default dir
        if !dirs.contains(&default) {
            dirs.push(default.clone());
        }
        // 3. Internal palettes dir (always, skip if same as default)
        if default != internal_palettes && !dirs.contains(&internal_palettes) {
            dirs.push(internal_palettes);
        }
        // 4. Extra dirs
        for d in extras {
            if !dirs.contains(&d) {
                dirs.push(d);
            }
        }
        dirs
    }

    // Add a directory to extra_dirs. Returns true if added, false if already present.
    pub fn add_extra_dir(&mut self, path: &str) -> bool {
        // Resolve path: if not absolute, resolve relative to home
        let resolved = helpers::resolve_home(path);

        // Reject default dir
        if self.default_dir_path().as_os_str() == resolved.as_str() {
            return false;
        }
        // Check extra_dirs
        if self.extra_dirs.contains(&resolved) {
            return false;
        }
        self.extra_dirs.push(resolved);
        true
    }
}

// JSON data structures
#[derive(Clone)]
pub struct ColourEntry {
    pub name: String,
    pub hex: String,
    pub hsl: String,
    pub rgb: String,
}

// PaletteFile
// Build a ColourEntry from a hex string, computing hsl/rgb from it.
fn entry_from_hex(name: &str, hex: &str) -> ColourEntry {
    let mut hex = hex.to_string();
    if !hex.starts_with('#') {
        hex = format!("#{hex}");
    }
    let c = hex_to_color(&hex);
    ColourEntry {
        name: name.to_string(),
        hex,
        hsl: format_hsl(&c),
        rgb: format_rgb(&c),
    }
}

// Build a ColourEntry from an rgb string like "rgb(30, 30, 46)".
fn entry_from_rgb(name: &str, rgb_str: &str) -> Option<ColourEntry> {
    let nums: Vec<u8> = rgb_str
        .trim_start_matches("rgb(")
        .trim_end_matches(')')
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if nums.len() != 3 {
        return None;
    }
    let c = Color::from_rgb(nums[0], nums[1], nums[2]);
    Some(ColourEntry {
        name: name.to_string(),
        hex: color_to_hex(&c),
        hsl: format_hsl(&c),
        rgb: rgb_str.to_string(),
    })
}

// Build a ColourEntry from an hsl string like "hsl(240, 21%, 14%)".
fn entry_from_hsl(name: &str, hsl_str: &str) -> Option<ColourEntry> {
    let parts: Vec<String> = hsl_str
        .trim_start_matches("hsl(")
        .trim_end_matches(')')
        .split(',')
        .map(|s| s.trim().replace('%', ""))
        .collect();
    if parts.len() != 3 {
        return None;
    }
    let hue = parts[0].parse::<f64>().ok()?;
    let sat = parts[1].parse::<f64>().ok()?;
    let lit = parts[2].parse::<f64>().ok()?;
    let c = Color::from_hsl(hue, sat / 100.0, lit / 100.0);
    Some(ColourEntry {
        name: name.to_string(),
        hex: color_to_hex(&c),
        hsl: hsl_str.to_string(),
        rgb: format_rgb(&c),
    })
}

// Escape a string for safe inclusion in a JSON double-quoted value.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

pub struct PaletteFile {
    pub path: PathBuf,
    pub name: String,
    pub source_dir: PathBuf,
    pub colours: Vec<ColourEntry>,
}

impl PaletteFile {
    pub fn load(path: PathBuf, source_dir: PathBuf) -> Result<Self, String> {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let val: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut colours = Vec::new();

        // Try map format: { "name": { hex, hsl, rgb } }
        if let Some(obj) = val.as_object() {
            for (ename, eval) in obj {
                let ename = helpers::sanitize_name(ename);
                let hex_val = eval["hex"].as_str();
                let hsl_val = eval["hsl"].as_str();
                let rgb_val = eval["rgb"].as_str();

                let entry = if let Some(h) = hex_val {
                    entry_from_hex(&ename, h)
                } else if let Some(r) = rgb_val {
                    entry_from_rgb(&ename, r).unwrap_or_else(|| entry_from_hex(&ename, "#000000"))
                } else if let Some(h) = hsl_val {
                    entry_from_hsl(&ename, h).unwrap_or_else(|| entry_from_hex(&ename, "#000000"))
                } else {
                    entry_from_hex(&ename, "#000000")
                };
                colours.push(entry);
            }
            return Ok(PaletteFile {
                path,
                name,
                source_dir,
                colours,
            });
        }

        // Try array format: [{ name, hex }] or [[name, hex]]
        if let Some(arr) = val.as_array() {
            for eval in arr {
                // Try object format: { "name": "x", "hex": "#xxx" }
                if let Some(obj) = eval.as_object() {
                    let ename = helpers::sanitize_name(
                        obj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown"),
                    );
                    let hex = obj.get("hex").and_then(|v| v.as_str()).unwrap_or("#000000");
                    colours.push(entry_from_hex(&ename, hex));
                    continue;
                }
                // Try tuple format: ["name", "#hex"]
                if let Some(tuple) = eval.as_array()
                    && tuple.len() >= 2
                {
                    let ename = helpers::sanitize_name(tuple[0].as_str().unwrap_or("unknown"));
                    let hex = tuple[1].as_str().unwrap_or("#000000");
                    colours.push(entry_from_hex(&ename, hex));
                }
            }
            return Ok(PaletteFile {
                path,
                name,
                source_dir,
                colours,
            });
        }

        Err(format!(
            "Failed to parse {}: unsupported format",
            path.display()
        ))
    }

    pub fn save(&self, dir_formats: &HashMap<String, Vec<String>>) -> Result<(), String> {
        // Determine which fields to save based on directory
        // Force hex-only for themes_dir
        let is_theme = self.source_dir == themes_dir();
        let src = self.source_dir.to_str().unwrap_or("");
        let fields = if is_theme {
            None // will make want() return true only for "hex"
        } else {
            // Try exact match first, then try with/without trailing slash
            dir_formats.get(src).or_else(|| {
                if let Some(stripped) = src.strip_suffix('/') {
                    dir_formats.get(stripped)
                } else {
                    dir_formats.get(&format!("{}/", src))
                }
            })
        };

        // Compute max width for each field across all colours
        let max_name = self
            .colours
            .iter()
            .map(|c| json_escape(&c.name).len())
            .max()
            .unwrap_or(0);
        let want = |f: &str| {
            if is_theme {
                f == "hex" // themes_dir always hex-only
            } else {
                fields.is_none_or(|v| v.contains(&f.to_string()))
            }
        };

        let max_hex_field = if want("hex") {
            self.colours
                .iter()
                .map(|c| format!("\"hex\": \"{}\"", c.hex).len())
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        let max_hsl_field = if want("hsl") {
            self.colours
                .iter()
                .map(|c| format!("\"hsl\": \"{}\"", c.hsl).len())
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        let max_rgb_field = if want("rgb") {
            self.colours
                .iter()
                .map(|c| format!("\"rgb\": \"{}\"", c.rgb).len())
                .max()
                .unwrap_or(0)
        } else {
            0
        };

        let mut lines = Vec::new();
        for c in &self.colours {
            let mut parts = Vec::new();

            if want("hex") {
                parts.push(format!(
                    "{:<width$}",
                    format!("\"hex\": \"{}\"", c.hex),
                    width = max_hex_field
                ));
            }
            if want("hsl") {
                parts.push(format!(
                    "{:<width$}",
                    format!("\"hsl\": \"{}\"", c.hsl),
                    width = max_hsl_field
                ));
            }
            if want("rgb") {
                parts.push(format!(
                    "{:<width$}",
                    format!("\"rgb\": \"{}\"", c.rgb),
                    width = max_rgb_field
                ));
            }

            let escaped_name = json_escape(&c.name);
            lines.push(format!(
                "  \"{}\":{:<pad$}{{ {} }}",
                escaped_name,
                "",
                parts.join(", "),
                pad = max_name - escaped_name.len() + 3,
            ));
        }

        let mut json = "{\n".to_string();
        for (i, line) in lines.iter().enumerate() {
            if i < lines.len() - 1 {
                json.push_str(line);
                json.push_str(",\n");
            } else {
                json.push_str(line);
                json.push('\n');
            }
        }
        json.push('}');

        fs::write(&self.path, json)
            .map_err(|e| format!("Failed to write {}: {}", self.path.display(), e))
    }
}

// Directory grouping
pub struct DirGroup {
    pub path: PathBuf,
    pub palette_indices: Vec<usize>,
    pub hidden_when_empty: bool,
}

// Directory scanning
// `hidden_dirs`: directories in this list get `hidden_when_empty = true` on
// their DirGroup, so the UI can hide them instead of showing "(empty)".
pub fn scan_directories(
    dirs: &[PathBuf],
    hidden_dirs: &[PathBuf],
) -> (Vec<PaletteFile>, Vec<DirGroup>, Vec<String>) {
    let mut palettes = Vec::new();
    let mut dir_groups = Vec::new();
    let mut warnings = Vec::new();

    for dir in dirs {
        let mut indices = Vec::new();
        if dir.is_dir()
            && let Ok(entries) = fs::read_dir(dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    match PaletteFile::load(path, dir.clone()) {
                        Ok(pf) => {
                            indices.push(palettes.len());
                            palettes.push(pf);
                        }
                        Err(e) => warnings.push(e),
                    }
                }
            }
        }
        dir_groups.push(DirGroup {
            path: dir.clone(),
            palette_indices: indices,
            hidden_when_empty: hidden_dirs.contains(dir),
        });
    }

    // Sort palettes alphabetically within each directory group
    for dg in &mut dir_groups {
        dg.palette_indices
            .sort_by_key(|&i| palettes[i].name.to_lowercase());
    }

    (palettes, dir_groups, warnings)
}

// Create a new empty palette file
pub fn create_palette(dir: &Path, name: &str) -> Result<PathBuf, String> {
    if !dir.is_dir() {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {}", dir.display(), e))?;
    }
    let path = dir.join(format!("{}.json", name));
    if path.exists() {
        return Err(format!(
            "Palette '{}' already exists at {}",
            name,
            path.display()
        ));
    }
    fs::write(&path, "{\n}\n").map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    Ok(path)
}

// Theme palette (palettes/theme.json)
const THEME_JSON: &str = r##"{
  "status-error":   { "hex": "#f38ba8" },
  "status-warn":    { "hex": "#fab387" },
  "action":         { "hex": "#a6e3a1" },
  "status-ok":      { "hex": "#94e2d5" },
  "input-text":     { "hex": "#94e2d5" },
  "border-unfocus": { "hex": "#89dceb" },
  "hotkey":         { "hex": "#89b4fa" },
  "input-fg":       { "hex": "#89b4fa" },
  "pointer-paired": { "hex": "#b4befe" },
  "border-focus":   { "hex": "#cba6f7" },
  "pointer":        { "hex": "#cba6f7" },
  "bg":             { "hex": ""        },
  "input-bg":       { "hex": ""        },
  "swatch-dark":    { "hex": "#303030" },
  "swatch-light":   { "hex": "#606060" },
  "path":           { "hex": "#6c7086" },
  "hotkey-sep":     { "hex": "#6c7086" },
  "empty":          { "hex": "#585b70" },
  "hint":           { "hex": "#7f849c" },
  "fg":             { "hex": "#cdd6f4" }
}
"##;

// Load or spawn the theme palette. Looks for `theme.json` in the config
// directory (~/.config/palette/themes/). If not found, creates it with defaults.
pub fn load_or_spawn_theme(config: &Config) -> (app::ThemeColors, Option<String>) {
    let theme_name = &config.theme_palette;

    // If theme_palette is an absolute path, use it directly
    // Otherwise, join with themes_dir()
    let theme_path = if Path::new(theme_name).is_absolute() {
        PathBuf::from(theme_name)
    } else {
        let dir = themes_dir();
        dir.join(theme_name)
    };

    if !theme_path.exists() {
        // Only create default file if it's in the themes directory
        if let Some(parent) = theme_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&theme_path, THEME_JSON);
    }

    let source_dir = theme_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    match PaletteFile::load(theme_path.clone(), source_dir) {
        Ok(pf) => (app::ThemeColors::from_palette(&pf), None),
        Err(e) => {
            let msg = format!(
                "{} is corrupted/malformed and could not be loaded: {}",
                theme_path.display(),
                e
            );
            eprintln!("Warning: {msg}");
            // Fall back to built-in default theme
            let fallback_path = std::env::temp_dir().join("palette-theme-fallback.json");
            let _ = fs::write(&fallback_path, THEME_JSON);
            let pf = PaletteFile::load(fallback_path, std::env::temp_dir())
                .expect("built-in THEME_JSON should always parse");
            (app::ThemeColors::from_palette(&pf), Some(msg))
        }
    }
}
