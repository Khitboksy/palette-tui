use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pastel::Color;

use crate::app::{App, InputMode, Mode, YesOrNoAction};
use crate::colour::*;
use crate::helpers;
use crate::palette;

use arboard::Clipboard;

/// Handle a key event. Returns true if the app should quit.
pub fn handle_key(key: KeyEvent, app: &mut App) -> bool {
    // Global: Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return true;
    }

    // If we are in a text-input prompt, handle it globally
    if let Some(ref input_mode) = app.input.mode.clone() {
        return match input_mode {
            InputMode::HexInput => handle_hex_input(app, key.code),
            InputMode::YesOrName { .. } => handle_yes_or_name(app, key.code),
            InputMode::YesOrNo { .. } => handle_yes_or_no(app, key.code),
            InputMode::ItemName => handle_item_name(app, key.code),
            InputMode::NewPaletteHex => handle_new_palette_hex(app, key.code),
            InputMode::AddDir => handle_add_dir(app, key.code),
            InputMode::Toggles { .. } => handle_toggles(app, key.code),
        };
    }

    match app.mode {
        Mode::Preview => handle_preview(app, key.code),
        Mode::Command => handle_command(app, key.code),
        Mode::Edit => handle_edit(app, key.code),
        Mode::PairSelect => handle_pair_select(app, key.code),
        Mode::PaletteSelect => handle_palette_select(app, key.code),
    }
}

// Preview mode -- j/k move, Enter selects, h/l switch palettes, q quits
fn handle_preview(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Up | KeyCode::Char('k') => {
            app.selected = app.selected.saturating_sub(1);
            app.select(app.selected);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.selected = (app.selected + 1).min(app.visible_len().saturating_sub(1));
            app.select(app.selected);
        }
        KeyCode::Enter => {
            app.select(app.selected);
            app.clear_status();
            app.mode = Mode::Command;
        }
        KeyCode::Char('h') | KeyCode::Left if app.palette.palettes.len() > 1 => {
            let idx = if app.palette.idx == 0 {
                app.palette.palettes.len() - 1
            } else {
                app.palette.idx - 1
            };
            app.load_palette(idx);
        }
        KeyCode::Char('l') | KeyCode::Right if app.palette.palettes.len() > 1 => {
            let idx = (app.palette.idx + 1) % app.palette.palettes.len();
            app.load_palette(idx);
        }
        KeyCode::Char('r') => {
            app.dirty = true;
            let hex = random_hex();
            app.current = hex_to_color(&hex);
            app.random_hex = hex.clone();
            app.edit.base_name = None;
            app.is_random = true;
            app.selected = 0;
            app.clear_status();
        }
        KeyCode::Char('.') => {
            app.reload_theme();
            app.set_status_ok("theme reloaded");
        }
        KeyCode::Char('e') => {
            app.begin_edit();
        }
        KeyCode::Tab => {
            app.palette.cursor = 0;
            app.palette.preview_idx = Some(app.palette.idx);
            app.mode = Mode::PaletteSelect;
        }
        _ => {}
    }
    false
}

// Command mode -- e, i, z, s/c/d, f, p/P
fn handle_command(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('z') | KeyCode::Esc => {
            app.mode = Mode::Preview;
        }
        KeyCode::Char('e') => {
            app.begin_edit();
        }
        KeyCode::Char('i') => {
            app.input.buf.clear();
            app.input.mode = Some(InputMode::HexInput);
        }
        KeyCode::Char('s') => {
            let hex = app.current_hex();
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(&hex).unwrap();
            app.set_status_ok(&format!("{hex} copied"));
        }
        KeyCode::Char('c') => {
            let text = format_rgb(&app.current);
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(&text).unwrap();
            app.set_status_ok(&format!("{text} copied"));
        }
        KeyCode::Char('d') => {
            let text = format_hsl(&app.current);
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(&text).unwrap();
            app.set_status_ok(&format!("{text} copied"));
        }
        KeyCode::Char('f') => {
            app.dirty = true;
            app.current = complement(&app.current);
            app.set_status_ok(&format!("flip {}", app.current_hex()));
        }
        // Quick-edit similar-to entries: 1-3 = named colours, 4-6 = palette
        KeyCode::Char(c @ '1'..='6') => {
            app.dirty = true;
            let idx = (c as usize) - ('1' as usize);
            if let Some((name, hex_val)) = app.similar_to.get(idx).cloned() {
                app.current = hex_to_color(&hex_val);
                app.edit.base_name = Some(name.clone());
                app.edit.colour_name = Some(name.clone());
                app.edit.clearing = false;
                app.clear_status();
                app.mode = Mode::Edit;
                // Warn for palette colours (indices 3-5 = keys 4-6)
                if idx >= 3
                    && let Some(palette) = app.palette.palettes.get(app.palette.idx)
                {
                    app.set_status_warn(&format!("{name} already exists in {}!", palette.name));
                }
            }
        }
        KeyCode::Char('p') => {
            let len = app.visible_len();
            if len > 0 {
                app.pair.cursor = app.selected.max(1) % len;
                app.mode = Mode::PairSelect;
            }
        }
        KeyCode::Char('P') => {
            app.pair.paired = None;
            app.pair.idx = None;
            app.pair.similar_name = None;
            app.pair.name.clear();
            app.set_status_ok("pair cleared");
        }
        KeyCode::Tab => {
            app.palette.cursor = 0;
            app.palette.preview_idx = Some(app.palette.idx);
            app.mode = Mode::PaletteSelect;
        }
        _ => {}
    }
    false
}

// Edit mode -- w (save prompt), e, z, p/P, r/R/g/G/b/B/j/J/l/k/K
fn handle_edit(app: &mut App, code: KeyCode) -> bool {
    app.dirty = true;
    let step = 0.01; // 1% of the 0..1 range for lighten/darken/saturate/desaturate
    let hue_step = 1.0 / 360.0; // 1 degree as a fraction of 360
    match code {
        KeyCode::Char('e') | KeyCode::Esc => {
            app.edit.clearing = false;
            app.edit.colour_name = None;
            app.mode = Mode::Command;
        }
        KeyCode::Char('z') => {
            app.mode = Mode::Preview;
        }
        KeyCode::Char('w') => {
            if let Some(ref name) = app.edit.colour_name.clone() {
                // Named colour from similar-to: skip name prompt, go to confirmation
                app.input.buf.clear();
                app.input.mode = Some(InputMode::YesOrNo {
                    prompt: format!("save {name} to palette? [y/n]: "),
                    action: YesOrNoAction::SaveNamed { name: name.clone() },
                });
            } else if let Some(ref name) = app.edit.base_name.clone() {
                app.input.buf.clear();
                app.input.mode = Some(InputMode::YesOrName { name: name.clone() });
            } else {
                app.input.buf.clear();
                app.input.mode = Some(InputMode::ItemName);
            }
        }
        KeyCode::Char('p') => {
            let len = app.visible_len();
            if len > 0 {
                app.pair.cursor = app.selected.max(1) % len;
                app.mode = Mode::PairSelect;
            }
        }
        KeyCode::Char('P') => {
            app.pair.paired = None;
            app.pair.idx = None;
            app.pair.similar_name = None;
            app.pair.name.clear();
            app.set_status_ok("pair cleared");
        }
        KeyCode::Char('c') => {
            // Clear colour: mark for empty hex on next write
            app.edit.clearing = true;
            app.set_status_warn("colour cleared -- press w to save");
        }
        // RGB channels -- extract, adjust, reconstruct
        KeyCode::Char('r') => adjust_rgb_channel(app, 'r', 1),
        KeyCode::Char('R') => adjust_rgb_channel(app, 'r', -1),
        KeyCode::Char('g') => adjust_rgb_channel(app, 'g', 1),
        KeyCode::Char('G') => adjust_rgb_channel(app, 'g', -1),
        KeyCode::Char('b') => adjust_rgb_channel(app, 'b', 1),
        KeyCode::Char('B') => adjust_rgb_channel(app, 'b', -1),
        // Hue rotation
        KeyCode::Char('j') => {
            app.current = rotate(&app.current, hue_step);
        }
        KeyCode::Char('J') => {
            app.current = rotate(&app.current, -hue_step);
        }
        // Lighten
        KeyCode::Char('l') => {
            app.current = lighten(&app.current, step);
        }
        KeyCode::Char('L') => {
            app.current = darken(&app.current, step);
        }

        // Saturate / desaturate
        KeyCode::Char('k') => {
            app.current = saturate(&app.current, step);
        }
        KeyCode::Char('K') => {
            app.current = desaturate(&app.current, step);
        }
        _ => {}
    }
    false
}

// PairSelect mode -- j/k move, Enter confirms, Esc cancels
fn handle_pair_select(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Command;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let len = app.visible_len();
            if len > 0 {
                app.pair.cursor = if app.pair.cursor == 0 {
                    len - 1
                } else {
                    app.pair.cursor - 1
                };
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let len = app.visible_len();
            if len > 0 {
                app.pair.cursor = (app.pair.cursor + 1) % len;
            }
        }
        KeyCode::Char(c @ '1'..='6') => {
            // Pair with similar-to entry
            let idx = (c as usize) - ('1' as usize);
            if let Some((name, hex_val)) = app.similar_to.get(idx).cloned() {
                let c = hex_to_color(&hex_val);
                app.pair.paired = Some(c);
                app.pair.idx = None;
                app.pair.similar_name = Some(name.clone());
                app.pair.name = name;
            }
            app.mode = Mode::Command;
        }
        KeyCode::Enter => {
            let idx = app.pair.cursor;
            if idx < app.visible_len() {
                let (name, hex) = app.visible_entry(idx);
                let name = name.to_string();
                let c = hex_to_color(hex);
                app.pair.paired = Some(c);
                app.pair.idx = Some(idx);
                app.pair.similar_name = None;
                app.pair.name = name;
            }
            app.mode = Mode::Command;
        }
        _ => {}
    }
    false
}

// PaletteSelect mode -- j/k move, Enter switches, n creates, a adds dir
// Get the total number of selectable items in palette select.
// This includes palette entries AND empty directory markers.
fn palette_select_len(app: &App) -> usize {
    let mut count = 0;
    for dg in &app.palette.dir_groups {
        if dg.palette_indices.is_empty() {
            count += 1; // empty dir marker
        } else {
            count += dg.palette_indices.len();
        }
    }
    count
}

/// Map a selectable index to a PaletteSelectItem
enum PaletteSelectItem {
    EmptyDir(PathBuf),
    Palette(usize), // index into app.palette.palettes
}

fn palette_select_item(app: &App, idx: usize) -> PaletteSelectItem {
    let mut count = 0;
    for dg in &app.palette.dir_groups {
        if dg.palette_indices.is_empty() {
            if count == idx {
                return PaletteSelectItem::EmptyDir(dg.path.clone());
            }
            count += 1;
        } else {
            for &pi in &dg.palette_indices {
                if count == idx {
                    return PaletteSelectItem::Palette(pi);
                }
                count += 1;
            }
        }
    }
    // Fallback
    PaletteSelectItem::EmptyDir(PathBuf::new())
}

fn handle_palette_select(app: &mut App, code: KeyCode) -> bool {
    let len = palette_select_len(app);
    match code {
        KeyCode::Esc | KeyCode::Tab => {
            app.palette.preview_idx = None;
            app.status.expiry = None;
            app.input.add_dir_retry = false;
            app.mode = Mode::Preview;
        }
        KeyCode::Up | KeyCode::Char('k') if len > 0 => {
            app.palette.cursor = if app.palette.cursor == 0 {
                len - 1
            } else {
                app.palette.cursor - 1
            };
            // Update preview
            if let PaletteSelectItem::Palette(idx) = palette_select_item(app, app.palette.cursor) {
                app.palette.preview_idx = Some(idx);
            }
        }
        KeyCode::Down | KeyCode::Char('j') if len > 0 => {
            app.palette.cursor = (app.palette.cursor + 1) % len;
            // Update preview
            if let PaletteSelectItem::Palette(idx) = palette_select_item(app, app.palette.cursor) {
                app.palette.preview_idx = Some(idx);
            }
        }
        KeyCode::Enter => {
            app.palette.preview_idx = None;
            // If there's a pending add_dir error, re-trigger the prompt
            if app.status.expiry.is_some() {
                app.clear_status();
                app.input.add_dir_retry = false;
                app.input.buf.clear();
                app.input.mode = Some(InputMode::AddDir);
            } else {
                match palette_select_item(app, app.palette.cursor) {
                    PaletteSelectItem::Palette(idx) => {
                        app.load_palette(idx);
                        app.mode = Mode::Preview;
                    }
                    PaletteSelectItem::EmptyDir(dir) => {
                        // Start new palette creation in an empty dir when selected
                        app.input.new_palette_dir = Some(dir);
                        app.input.buf.clear();
                        app.input.mode = Some(InputMode::ItemName);
                    }
                }
            }
        }
        KeyCode::Char('n') => {
            // Create a new palette and ask for directory first
            // Default to the directory of the currently highlighted palette
            let dir = match palette_select_item(app, app.palette.cursor) {
                PaletteSelectItem::Palette(idx) => app.palette.palettes[idx].source_dir.clone(),
                PaletteSelectItem::EmptyDir(dir) => dir,
            };
            app.input.new_palette_dir = Some(dir);
            app.palette.preview_idx = None;
            app.input.buf.clear();
            app.input.mode = Some(InputMode::ItemName);
        }
        KeyCode::Char('a') => {
            // Add a new directory
            app.input.buf.clear();
            app.input.mode = Some(InputMode::AddDir);
        }
        KeyCode::Char('f') => {
            // Set dir_formats for the highlighted directory
            let dir = match palette_select_item(app, app.palette.cursor) {
                PaletteSelectItem::Palette(idx) => app.palette.palettes[idx].source_dir.clone(),
                PaletteSelectItem::EmptyDir(dir) => dir,
            };
            // Read current formats from config (default all on)
            let formats = app
                .palette
                .config
                .dir_formats
                .get(dir.to_str().unwrap_or_default())
                .cloned()
                .unwrap_or_else(|| vec!["hex".to_string(), "hsl".to_string(), "rgb".to_string()]);
            app.input.format_focused = 0;
            app.input.mode = Some(InputMode::Toggles {
                dir,
                hex: formats.contains(&"hex".to_string()),
                hsl: formats.contains(&"hsl".to_string()),
                rgb: formats.contains(&"rgb".to_string()),
            });
        }
        _ => {}
    }
    false
}

// Input handlers

/// Returns true if a character is allowed in the current input mode.
/// Each mode has its own whitelist to prevent invalid characters upfront.
fn is_char_allowed(c: char, mode: &InputMode, buf_len: usize) -> bool {
    match mode {
        InputMode::YesOrNo { .. } => matches!(c, 'y' | 'Y' | 'n' | 'N'),
        InputMode::YesOrName { .. } => {
            c == 'y' || c == 'Y' || c.is_ascii_alphanumeric() || c == '-' || c == '_'
        }
        InputMode::ItemName => c.is_ascii_alphanumeric() || c == '-' || c == '_',
        InputMode::Toggles { .. } => false,
        InputMode::AddDir => {
            c.is_ascii_alphanumeric()
                || c == '~'
                || c == '.'
                || c == '/'
                || c == '-'
                || c == '_'
                || c == ' '
        }
        InputMode::HexInput | InputMode::NewPaletteHex => {
            (c == '#' || c.is_ascii_hexdigit()) && buf_len < 7
        }
    }
}

/// Handle the common text-input keys (Esc, Backspace, Char).
/// Returns `Some(Enter)` if the key is Enter -- the caller handles it.
/// Returns `None` for all other keys (already handled here).
fn handle_text_input(app: &mut App, code: KeyCode) -> Option<KeyCode> {
    let mode = app.input.mode.clone();
    let mode_ref = mode.as_ref().unwrap();
    match code {
        KeyCode::Esc => {
            app.input.mode = None;
            app.input.buf.clear();
            None
        }
        KeyCode::Enter => Some(KeyCode::Enter),
        KeyCode::Backspace => {
            app.input.buf.pop();
            None
        }
        KeyCode::Char(c) => {
            if is_char_allowed(c, mode_ref, app.input.buf.len()) {
                app.input.buf.push(c);
            } else {
                let display = match c {
                    ' ' => "space".to_string(),
                    c if c.is_control() => format!("{:?}", c),
                    _ => c.to_string(),
                };
                app.set_status_warn(&format!("'{display}' is not a valid character"));
                app.status.expiry = Some(Instant::now() + Duration::from_secs(3));
            }
            None
        }
        _ => None,
    }
}

/// Adjust a single RGB channel by `delta` (+1 or -1).
fn adjust_rgb_channel(app: &mut App, channel: char, delta: i16) {
    let rgba = app.current.to_rgba();
    let (r, g, b) = match channel {
        'r' => (rgba.r as i16 + delta, rgba.g as i16, rgba.b as i16),
        'g' => (rgba.r as i16, rgba.g as i16 + delta, rgba.b as i16),
        'b' => (rgba.r as i16, rgba.g as i16, rgba.b as i16 + delta),
        _ => return,
    };
    app.current = Color::from_rgb(
        r.clamp(0, 255) as u8,
        g.clamp(0, 255) as u8,
        b.clamp(0, 255) as u8,
    );
}

fn handle_hex_input(app: &mut App, code: KeyCode) -> bool {
    if let Some(KeyCode::Enter) = handle_text_input(app, code) {
        app.dirty = true;
        let hex = app.input.buf.trim().to_string();
        let clean = normalize_hex(&hex);
        if let Some(clean) = clean {
            app.current = hex_to_color(&clean);
            app.pair.paired = None;
            app.pair.idx = None;
            app.edit.base_name = None;
            app.input.mode = None;
            app.input.buf.clear();
            app.mode = Mode::Edit;
        } else {
            app.set_status_error("invalid hex");
            app.input.mode = None;
            app.input.buf.clear();
        }
    }
    false
}

fn handle_yes_or_name(app: &mut App, code: KeyCode) -> bool {
    if let Some(KeyCode::Enter) = handle_text_input(app, code) {
        let answer = app.input.buf.trim().to_string();
        let save_name = if helpers::is_yes(&answer) {
            match &app.input.mode {
                Some(InputMode::YesOrName { name }) => name.clone(),
                _ => unreachable!(),
            }
        } else if !answer.is_empty() {
            answer
        } else {
            app.input.mode = None;
            app.input.buf.clear();
            return false;
        };
        app.write_colour_to_palette(&save_name);
    }
    false
}

fn handle_yes_or_no(app: &mut App, code: KeyCode) -> bool {
    if let Some(KeyCode::Enter) = handle_text_input(app, code) {
        let answer = app.input.buf.trim().to_string();
        if helpers::is_yes(&answer) {
            if let Some(InputMode::YesOrNo { action, .. }) = &app.input.mode {
                match action {
                    YesOrNoAction::SaveNamed { name } => {
                        let name = name.clone();
                        app.write_colour_to_palette(&name);
                        app.edit.colour_name = None;
                    }
                    // DeleteColour and DeletePalette handled in later steps
                    _ => {}
                }
            }
        }
        app.input.mode = None;
        app.input.buf.clear();
    }
    false
}

/// Generic item name handler (alphanumeric, -, _). Used for colour names,
/// palette names, or any other named item. Validates and returns the name
/// through the buffer -- callers inspect the buffer after Enter.
fn handle_item_name(app: &mut App, code: KeyCode) -> bool {
    if let Some(KeyCode::Enter) = handle_text_input(app, code) {
        let name = app.input.buf.trim().to_string();
        if name.is_empty() {
            app.set_status_error("name cannot be empty");
            app.input.mode = None;
            app.input.buf.clear();
            return false;
        }
        if !helpers::validate_name(&name) {
            app.set_status_error("only letters, numbers, - and _ allowed");
            return false;
        }
        // If there's a pending palette creation dir, this is a palette name
        if app.input.new_palette_dir.is_some() {
            let dir = app
                .input
                .new_palette_dir
                .clone()
                .unwrap_or_else(|| app.palette.config.default_dir_path());
            match palette::create_palette(&dir, &name) {
                Ok(path) => {
                    app.rescan();
                    if let Some(pos) = app.palette.palettes.iter().position(|pf| pf.path == path) {
                        app.load_palette(pos);
                        app.palette.cursor = pos;
                    }
                    app.input.new_palette_hex = None;
                    app.input.buf.clear();
                    app.input.mode = Some(InputMode::NewPaletteHex);
                    app.set_status_ok(&format!("created {name}"));
                }
                Err(e) => {
                    app.set_status_error(&e);
                    app.input.mode = None;
                    app.input.buf.clear();
                    app.input.new_palette_dir = None;
                }
            }
        } else if let Some(hex) = app.input.new_palette_hex.take() {
            // Colour name for new palette
            app.dirty = true;
            app.current = hex_to_color(&hex);
            app.is_random = false;
            app.write_colour_to_palette(&name);
            let idx = app.palette.idx;
            app.load_palette(idx);
            app.mode = Mode::Preview;
            app.input.mode = None;
            app.input.buf.clear();
            app.input.new_palette_dir = None;
        } else {
            // Plain colour name (save new in edit mode)
            app.write_colour_to_palette(&name);
            app.input.mode = None;
            app.input.buf.clear();
        }
    }
    false
}

// New palette creation flow
fn handle_new_palette_hex(app: &mut App, code: KeyCode) -> bool {
    // Extra Esc cleanup for this handler
    if code == KeyCode::Esc {
        app.input.new_palette_hex = None;
        app.input.new_palette_dir = None;
    }
    if let Some(KeyCode::Enter) = handle_text_input(app, code) {
        let hex = app.input.buf.trim().to_string();
        if let Some(clean) = normalize_hex(&hex) {
            app.input.new_palette_hex = Some(clean);
            app.input.buf.clear();
            app.input.mode = Some(InputMode::ItemName);
        } else {
            app.set_status_error("invalid hex");
            app.input.buf.clear();
        }
    }
    false
}

// Add directory flow
fn handle_add_dir(app: &mut App, code: KeyCode) -> bool {
    if let Some(KeyCode::Enter) = handle_text_input(app, code) {
        let dir = app.input.buf.trim().to_string();
        if dir.is_empty() {
            app.input.mode = None;
            app.input.buf.clear();
            return false;
        }
        if !helpers::validate_dir_path(&dir) {
            app.set_status_error("path: only letters, numbers, /, ., -, _, ~, and spaces allowed");
            return false;
        }
        let resolved = helpers::resolve_home(&dir);
        let path = PathBuf::from(&resolved);
        if !path.is_dir() {
            app.set_status_warn(&format!("'{}' is not a directory", resolved));
            app.status.expiry = Some(Instant::now() + Duration::from_secs(5));
            app.input.add_dir_retry = true;
            app.input.mode = None;
            app.input.buf.clear();
            return false;
        }
        if app.palette.config.add_extra_dir(&resolved) {
            match app.palette.config.save() {
                Ok(()) => {
                    app.rescan();
                    app.set_status_ok(&format!("added {}", resolved));
                    app.status.expiry = None;
                    app.input.add_dir_retry = false;
                }
                Err(e) => {
                    // Revert the in-memory change
                    app.palette.config.extra_dirs.retain(|d| d != &resolved);
                    app.set_status_error(&e);
                }
            }
        } else {
            app.set_status_warn(&format!("'{}' is already configured", resolved));
            app.status.expiry = None;
            app.input.add_dir_retry = false;
        }
        app.input.mode = None;
        app.input.buf.clear();
    }
    false
}

// Generic toggle selection flow -- h/l or arrows to move, space to toggle, Enter to confirm
fn handle_toggles(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => {
            app.input.mode = None;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.input.format_focused = if app.input.format_focused == 0 {
                2
            } else {
                app.input.format_focused - 1
            };
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.input.format_focused = (app.input.format_focused + 1) % 3;
        }
        KeyCode::Char(' ') => {
            if let Some(InputMode::Toggles {
                ref mut hex,
                ref mut hsl,
                ref mut rgb,
                ..
            }) = app.input.mode
            {
                match app.input.format_focused {
                    0 => *hex = !*hex,
                    1 => *hsl = !*hsl,
                    2 => *rgb = !*rgb,
                    _ => {}
                }
            }
        }
        KeyCode::Enter => {
            if let Some(InputMode::Toggles {
                ref dir,
                hex,
                hsl,
                rgb,
            }) = app.input.mode.clone()
            {
                if hex || hsl || rgb {
                    let dir_str = dir.to_string_lossy().to_string();
                    if hex && hsl && rgb {
                        app.palette.config.dir_formats.remove(&dir_str);
                    } else {
                        let mut formats = Vec::new();
                        if hex {
                            formats.push("hex".to_string());
                        }
                        if hsl {
                            formats.push("hsl".to_string());
                        }
                        if rgb {
                            formats.push("rgb".to_string());
                        }
                        app.palette.config.dir_formats.insert(dir_str, formats);
                    }
                    match app.palette.config.save() {
                        Ok(()) => {
                            app.set_status_ok("formats updated");
                            app.status.expiry = None;
                        }
                        Err(e) => {
                            app.set_status_error(&e);
                        }
                    }
                    app.input.mode = None;
                } else {
                    app.set_status_warn("at least one format required");
                }
            }
        }
        _ => {}
    }
    false
}
