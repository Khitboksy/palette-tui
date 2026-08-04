use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use pastel::Color;
use ratatui::style::Color as RatatuiColor;

use crate::colour::*;
use crate::palette::{self, ColourEntry, Config, DirGroup, PaletteFile};

// Status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Ok,
    Warn,
    Error,
}

// Theme colours
pub struct ThemeColors {
    pub bg: RatatuiColor,
    pub fg: RatatuiColor,
    pub border_focus: RatatuiColor,
    pub border_unfocus: RatatuiColor,
    pub status_ok: RatatuiColor,
    pub status_warn: RatatuiColor,
    pub status_error: RatatuiColor,
    pub path: RatatuiColor,
    pub empty: RatatuiColor,
    pub swatch_light: RatatuiColor,
    pub swatch_dark: RatatuiColor,
    pub hint: RatatuiColor,
    pub hotkey: RatatuiColor,
    pub hotkey_sep: RatatuiColor,
    pub input_bg: RatatuiColor,
    pub input_fg: RatatuiColor,
    pub input_text: RatatuiColor,
    pub action: RatatuiColor,
    pub pointer: RatatuiColor,
    pub pointer_paired: RatatuiColor,
}

impl ThemeColors {
    /// Parse a hex string like "#cba6f7" into a ratatui Color.
    /// Returns Color::Reset for empty strings (meaning "no colour").
    fn from_hex(hex: &str) -> RatatuiColor {
        let h = hex.trim_start_matches('#');
        if h.is_empty() {
            return RatatuiColor::Reset;
        }
        if h.len() == 6
            && let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&h[0..2], 16),
                u8::from_str_radix(&h[2..4], 16),
                u8::from_str_radix(&h[4..6], 16),
            )
        {
            return RatatuiColor::Rgb(r, g, b);
        }
        RatatuiColor::White // fallback
    }

    /// Build from a loaded palette file (theme.json)
    pub fn from_palette(palette: &PaletteFile) -> Self {
        let get = |name: &str, default: &str| -> RatatuiColor {
            palette
                .colours
                .iter()
                .find(|ce| ce.name == name)
                .map(|ce| Self::from_hex(&ce.hex))
                .unwrap_or_else(|| Self::from_hex(default))
        };
        Self {
            bg: get("bg", ""),
            fg: get("fg", "#cdd6f4"),
            border_focus: get("border-focus", "#cba6f7"),
            border_unfocus: get("border-unfocus", "#89b4fa"),
            status_ok: get("status-ok", "#94e2d5"),
            status_warn: get("status-warn", "#fab387"),
            status_error: get("status-error", "#f38ba8"),
            path: get("path", "#6c7086"),
            empty: get("empty", "#585b70"),
            swatch_light: get("swatch-light", "#606060"),
            swatch_dark: get("swatch-dark", "#303030"),
            hint: get("hint", "#7f849c"),
            hotkey: get("hotkey", "#89b4fa"),
            hotkey_sep: get("hotkey-sep", "#6c7086"),
            input_bg: get("input-bg", "#313244"),
            input_fg: get("input-fg", "#cdd6f4"),
            input_text: get("input-text", "#94e2d5"),
            action: get("action", "#cba6f7"),
            pointer: get("pointer", "#cba6f7"),
            pointer_paired: get("pointer-paired", "#89b4fa"),
        }
    }
}

// App state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Preview,
    Command,
    Edit,
    PairSelect,
    PaletteSelect,
}

// Input types
#[derive(Debug, Clone)]
pub enum InputMode {
    // Generic Input Modes
    YesOrNo {
        prompt: String,
        action: YesOrNoAction,
    },
    YesOrName {
        name: String,
    },
    ItemName,
    Toggles {
        dir: PathBuf,
        hex: bool,
        hsl: bool,
        rgb: bool,
    },

    // Flow-specific modes
    HexInput,
    NewPaletteHex,
    AddDir,
}

/// Payload carried by `YesOrNo` to distinguish what happens on confirm.
#[derive(Debug, Clone)]
pub enum YesOrNoAction {
    /// Save a colour under this name (replaces old SaveNamed)
    SaveNamed { name: String },
    /// Delete a colour from a palette
    DeleteColour {
        colour_name: String,
        palette_idx: usize,
    },
    /// Delete an entire palette file
    DeletePalette { palette_idx: usize },
}

// Input attributes
pub struct InputState {
    pub mode: Option<InputMode>,
    pub buf: String,
    pub new_palette_dir: Option<PathBuf>,
    pub new_palette_hex: Option<String>,
    pub add_dir_retry: bool,
    pub format_focused: usize, // 0=hex, 1=hsl, 2=rgb
}

// Pair attributes
pub struct PairState {
    pub cursor: usize,
    pub paired: Option<Color>,
    pub idx: Option<usize>,
    pub name: String,
    pub similar_name: Option<String>,
}

// Edit attributes
pub struct EditState {
    pub base_name: Option<String>,
    pub colour_name: Option<String>,
    pub clearing: bool,
}

// Palette attributes
pub struct PaletteState {
    pub palettes: Vec<PaletteFile>,
    pub dir_groups: Vec<DirGroup>,
    pub config: Config,
    pub idx: usize,
    pub cursor: usize,
    pub preview_idx: Option<usize>,
    pub show_hidden: bool,
}

// Status attributes
pub struct StatusState {
    pub msg: String,
    pub level: StatusLevel,
    pub expiry: Option<Instant>,
}

pub struct App {
    pub colours: Vec<(String, String)>, // (name, hex)
    pub selected: usize,
    pub mode: Mode,
    pub current: Color,
    pub is_random: bool,
    pub random_hex: String,
    pub current_empty: bool,
    pub similar_to: Vec<(String, String)>,
    pub theme: ThemeColors,
    pub dirty: bool,
    pub input: InputState,
    pub pair: PairState,
    pub edit: EditState,
    pub palette: PaletteState,
    pub status: StatusState,
    pub hidden_dirs: HashSet<PathBuf>,
}

impl App {
    pub fn begin_edit(&mut self) {
        if self.is_random {
            self.edit.base_name = None;
        } else if self.selected < self.visible_len() && self.selected < self.colours.len() {
            self.edit.base_name = Some(self.colours[self.selected].0.clone());
        } else {
            self.edit.base_name = None;
        }
        self.edit.clearing = false;
        self.clear_status();
        self.mode = Mode::Edit;
    }
    pub fn new(config: Config) -> Self {
        // Load or spawn theme first (may create themes_dir and theme.json)
        let (theme, theme_err) = palette::load_or_spawn_theme(&config);

        // Now scan directories (theme.json exists if it was just created)
        let mut dirs = config.all_dirs();

        // Dev-only: append repo palettes dir as its own scan group
        if std::env::var("DEV_OPTIONS").unwrap_or_default() == "1" {
            let repo_palettes = std::path::PathBuf::from("palettes");
            if repo_palettes.is_dir() && !dirs.contains(&repo_palettes) {
                dirs.push(repo_palettes);
            }
        }

        let (palettes, mut dir_groups, scan_warnings) =
            palette::scan_directories(&dirs, &[palette::default_palettes()]);

        // In dev mode, mark protected dirs as non-hideable (always visible)
        if std::env::var("DEV_OPTIONS").unwrap_or_default() == "1" {
            let themes = palette::themes_dir();
            let internal = palette::default_palettes();
            let repo_palettes = std::path::PathBuf::from("palettes");
            for dg in &mut dir_groups {
                if dg.path == themes || dg.path == internal || dg.path == repo_palettes {
                    dg.hideable = false;
                }
            }
        }

        // Find the initial palette: match by name if default.palette is set,
        // otherwise the first palette from the default dir.
        let default_dir = config.default_dir_path();
        let initial_idx = config
            .default_palette_name()
            .and_then(|name| {
                palettes.iter().position(|pf| {
                    pf.source_dir == default_dir
                        && pf.path.file_stem().and_then(|s| s.to_str()) == Some(name)
                })
            })
            .or_else(|| {
                dir_groups
                    .iter()
                    .find(|dg| dg.path == default_dir)
                    .and_then(|dg| dg.palette_indices.first().copied())
            })
            .or_else(|| {
                palettes
                    .iter()
                    .position(|p| p.source_dir != palette::themes_dir())
            })
            .unwrap_or(0);

        let mut app = Self {
            colours: Vec::new(),
            selected: 0,
            mode: Mode::Preview,
            current: Color::black(),
            is_random: false,
            random_hex: String::new(),
            current_empty: false,
            similar_to: Vec::new(),
            theme,
            dirty: false,
            input: InputState {
                mode: None,
                buf: String::new(),
                new_palette_dir: None,
                new_palette_hex: None,
                add_dir_retry: false,
                format_focused: 0,
            },
            pair: PairState {
                cursor: 0,
                paired: None,
                idx: None,
                name: String::new(),
                similar_name: None,
            },
            edit: EditState {
                base_name: None,
                colour_name: None,
                clearing: false,
            },
            palette: PaletteState {
                palettes,
                dir_groups,
                config,
                idx: initial_idx,
                cursor: 0,
                preview_idx: None,
                show_hidden: false,
            },
            status: StatusState {
                msg: String::new(),
                level: StatusLevel::Ok,
                expiry: None,
            },
            hidden_dirs: palette::load_hidden_dirs(),
        };
        // Apply persisted hidden state to dir_groups
        for dg in &mut app.palette.dir_groups {
            if app.hidden_dirs.contains(&dg.path) {
                dg.hidden = true;
            }
        }
        app.load_palette(initial_idx);
        if let Some(msg) = theme_err {
            app.set_status_warn(&msg);
        }
        for msg in scan_warnings {
            app.set_status_warn(&msg);
        }
        app
    }

    // Status helpers
    pub fn set_status(&mut self, msg: &str, level: StatusLevel) {
        self.status.msg = msg.to_string();
        self.status.level = level;
    }
    pub fn set_status_ok(&mut self, msg: &str) {
        self.set_status(msg, StatusLevel::Ok);
    }
    pub fn set_status_warn(&mut self, msg: &str) {
        self.set_status(msg, StatusLevel::Warn);
    }
    pub fn set_status_error(&mut self, msg: &str) {
        self.set_status(msg, StatusLevel::Error);
    }
    pub fn clear_status(&mut self) {
        self.status.msg.clear();
        self.status.level = StatusLevel::Ok;
        self.status.expiry = None;
    }

    /// Return the ratatui colour for the current status level.
    pub fn status_color(&self) -> RatatuiColor {
        match self.status.level {
            StatusLevel::Ok => self.theme.status_ok,
            StatusLevel::Warn => self.theme.status_warn,
            StatusLevel::Error => self.theme.status_error,
        }
    }

    /// Compute the "similar to" list: 3 closest named colours + 3 closest
    /// palette entries, all deduplicated by hex value.
    pub fn compute_similar_to(&mut self) {
        let colour = &self.current;
        let hex = self.current_hex();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut result: Vec<(String, String)> = Vec::new();

        // Column 1: closest CSS/X11 named colours (up to 3)
        let named = closest_named_colors(colour, 3);
        for (name, color) in &named {
            let h = color_to_hex(color);
            if !seen.contains(&h) && h != hex {
                seen.insert(h.clone());
                result.push((name.to_string(), h));
            }
            if result.len() >= 3 {
                break;
            }
        }

        // Column 2: closest palette entries (up to 3, deduplicated)
        let mut distances: Vec<(f64, String, String)> = self
            .colours
            .iter()
            .filter(|(_, h)| {
                let clean = h.trim_start_matches('#').to_string();
                !clean.is_empty() && *h != hex && !seen.contains(h)
            })
            .map(|(name, h)| {
                let c = hex_to_color(h);
                let dist = colour.distance_delta_e_ciede2000(&c);
                (dist, name.clone(), h.clone())
            })
            .collect();
        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_dist, name, h) in distances.into_iter().take(3) {
            if !seen.contains(&h) {
                seen.insert(h.clone());
                result.push((name, h));
            }
        }

        self.similar_to = result;
    }

    /// Rescan all directories (after adding a dir or creating a palette)
    pub fn rescan(&mut self) {
        let dirs = self.palette.config.all_dirs();
        let (palettes, dir_groups, scan_warnings) =
            palette::scan_directories(&dirs, &[palette::default_palettes()]);
        self.palette.palettes = palettes;
        self.palette.dir_groups = dir_groups;
        // Re-apply user-hidden dirs
        for dg in &mut self.palette.dir_groups {
            if self.hidden_dirs.contains(&dg.path) {
                dg.hidden = true;
            }
        }
        for msg in scan_warnings {
            self.set_status_warn(&msg);
        }
        // Clamp palette idx
        if self.palette.idx >= self.palette.palettes.len() && !self.palette.palettes.is_empty() {
            self.palette.idx = self.palette.palettes.len() - 1;
        }
    }

    /// Reload theme from theme.json
    pub fn reload_theme(&mut self) {
        let (theme, theme_err) = palette::load_or_spawn_theme(&self.palette.config);
        self.theme = theme;
        if let Some(msg) = theme_err {
            self.set_status_warn(&msg);
        }
    }

    pub fn select(&mut self, idx: usize) {
        self.dirty = true;
        self.selected = idx;
        self.clear_status();
        if self.is_random && idx == 0 {
            self.current = hex_to_color(&self.random_hex);
            self.current_empty = false;
        } else {
            let real_idx = if self.is_random { idx - 1 } else { idx };
            if real_idx < self.colours.len() {
                let hex = &self.colours[real_idx].1;
                self.current_empty = hex.trim_start_matches('#').is_empty();
                self.current = hex_to_color(hex);
            }
        }
    }

    pub fn current_hex(&self) -> String {
        color_to_hex(&self.current)
    }

    /// Number of visible entries in the list (including virtual random entry)
    pub fn visible_len(&self) -> usize {
        self.colours.len() + if self.is_random { 1 } else { 0 }
    }

    /// Get name and hex for a visible index (accounting for virtual random entry)
    pub fn visible_entry(&self, idx: usize) -> (&str, &str) {
        if self.is_random && idx == 0 {
            (&self.random_hex, &self.random_hex)
        } else {
            let real_idx = if self.is_random { idx - 1 } else { idx };
            if real_idx < self.colours.len() {
                (&self.colours[real_idx].0, &self.colours[real_idx].1)
            } else {
                ("", "")
            }
        }
    }

    pub fn load_palette(&mut self, idx: usize) {
        if idx < self.palette.palettes.len() {
            self.dirty = true;
            self.palette.idx = idx;
            self.colours.clear();
            for ce in &self.palette.palettes[idx].colours {
                // Preserve original hex (empty stays empty)
                self.colours.push((ce.name.clone(), ce.hex.clone()));
            }
            self.selected = 0;
            self.current_empty = false;
            if !self.colours.is_empty() {
                self.current_empty = self.colours[0].1.trim_start_matches('#').is_empty();
                self.current = hex_to_color(&self.colours[0].1);
            }
            self.pair.paired = None;
            self.pair.idx = None;
            self.pair.cursor = 0;
            self.is_random = false;
            self.random_hex.clear();
            self.clear_status();
        }
    }

    /// Clear input mode and buffer -- the common "dismiss prompt" action.
    pub fn reset_input(&mut self) {
        self.input.mode = None;
        self.input.buf.clear();
    }

    pub fn write_colour_to_palette(&mut self, name: &str) {
        if self.palette.idx >= self.palette.palettes.len() {
            self.set_status_error("No palette loaded");
            return;
        }
        let clearing = self.edit.clearing;
        let entry = if clearing {
            // Write empty hex (no colour)
            ColourEntry {
                name: name.to_string(),
                hex: String::new(),
                hsl: String::new(),
                rgb: String::new(),
            }
        } else {
            let hex = self.current_hex();
            ColourEntry {
                name: name.to_string(),
                hex: hex.clone(),
                hsl: format_hsl(&self.current),
                rgb: format_rgb(&self.current),
            }
        };
        let pal = &mut self.palette.palettes[self.palette.idx];

        // Random entries always create new (never overwrite)
        let existing = if self.is_random {
            None
        } else {
            pal.colours.iter().position(|ce| ce.name == name)
        };
        if let Some(idx) = existing {
            pal.colours[idx] = entry;
        } else {
            pal.colours.push(entry);
        }

        match pal.save(&self.palette.config.dir_formats) {
            Ok(()) => {
                self.edit.clearing = false;
                // Reload palette to update the colour list
                let idx = self.palette.idx;
                self.load_palette(idx);
                // Find and select the saved colour
                if let Some(pos) = self.colours.iter().position(|c| c.0 == name) {
                    self.selected = pos;
                    self.current = hex_to_color(&self.colours[pos].1);
                }
                self.mode = Mode::Command;
                self.set_status_ok(&format!("{name} written"));
            }
            Err(e) => {
                self.set_status_error(&format!("failed to save \"{name}\": {e}"));
            }
        }
    }

    pub fn pair_clear(&mut self) {
        self.pair.paired = None;
        self.pair.idx = None;
        self.pair.similar_name = None;
        self.pair.name.clear();
        self.set_status_ok("pair cleared");
    }
    pub fn pair_select(&mut self) {
        let len = self.visible_len();
        if len > 0 {
            self.pair.cursor = self.selected.max(1) % len;
            self.mode = Mode::PairSelect;
        }
    }

    pub fn cursor_dir(&self) -> PathBuf {
        match crate::input::palette_select_item(self, self.palette.cursor) {
            crate::input::PaletteSelectItem::Palette(idx) => {
                self.palette.palettes[idx].source_dir.clone()
            }
            crate::input::PaletteSelectItem::EmptyDir(dir) => dir,
        }
    }
    pub fn clamp_palette_cursor(&mut self) {
        let len = crate::input::palette_select_len(&self);
        if self.palette.cursor >= len && len > 0 {
            self.palette.cursor = len - 1;
        }
    }
}
