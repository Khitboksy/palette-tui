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
    if let Some(input_mode) = &app.input.mode {
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

fn enter_palette_select(app: &mut App) {
    app.palette.cursor = palette_idx_to_cursor(app, app.palette.idx);
    app.palette.preview_idx = Some(app.palette.idx);
    app.mode = Mode::PaletteSelect;
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
        KeyCode::Char('D') if !app.is_random && app.selected < app.visible_len() => {
            let (name, _) = app.visible_entry(app.selected);
            let colour_name = name.to_string();
            let palette_idx = app.palette.idx;
            let palette_name = app.palette.palettes[palette_idx].name.clone();
            app.input.buf.clear();
            app.input.mode = Some(InputMode::YesOrNo {
                prompt: format!("delete {colour_name} from {palette_name}? [y/n]: "),
                action: YesOrNoAction::DeleteColour {
                    colour_name,
                    palette_idx,
                },
            });
        }
        KeyCode::Tab => {
            enter_palette_select(app);
        }
        _ => {}
    }
    false
}

fn copy_to_clipboard(app: &mut App, text: &str) {
    match Clipboard::new() {
        Ok(mut cb) => match cb.set_text(text) {
            Ok(()) => app.set_status_ok(&format!("{text} copied")),
            Err(e) => app.set_status_error(&format!("clipboard error: {e}")),
        },
        Err(e) => app.set_status_error(&format!("clipboard unavailable: {e}")),
    }
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
            copy_to_clipboard(app, &hex);
        }
        KeyCode::Char('c') => {
            let text = format_rgb(&app.current);
            copy_to_clipboard(app, &text);
        }
        KeyCode::Char('d') => {
            let text = format_hsl(&app.current);
            copy_to_clipboard(app, &text);
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
            app.pair_select();
        }
        KeyCode::Char('P') => {
            app.pair_clear();
        }
        KeyCode::Tab => {
            enter_palette_select(app);
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
            app.clear_status();
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
            app.pair_select();
        }
        KeyCode::Char('P') => {
            app.pair_clear();
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

/// Returns true if a directory group should be hidden from the palette select list.
pub fn dir_hidden(dg: &crate::palette::DirGroup, show_hidden: bool) -> bool {
    if show_hidden {
        return false;
    }
    dg.hidden || (dg.hidden_when_empty && dg.palette_indices.is_empty())
}

/// Map a selectable index in palette select to a PaletteSelectItem
pub enum PaletteSelectItem {
    EmptyDir(PathBuf),
    Palette(usize), // index into app.palette.palettes
}

/// Collect all visible selectable items in palette select order.
fn visible_items(app: &App) -> Vec<PaletteSelectItem> {
    let mut items = Vec::new();
    for dg in &app.palette.dir_groups {
        if dir_hidden(dg, app.palette.show_hidden) {
            continue;
        }
        if dg.palette_indices.is_empty() {
            items.push(PaletteSelectItem::EmptyDir(dg.path.clone()));
        } else {
            for &pi in &dg.palette_indices {
                items.push(PaletteSelectItem::Palette(pi));
            }
        }
    }
    items
}

pub fn palette_select_item(app: &App, idx: usize) -> PaletteSelectItem {
    visible_items(app)
        .into_iter()
        .nth(idx)
        .unwrap_or(PaletteSelectItem::EmptyDir(PathBuf::new()))
}

pub fn palette_select_len(app: &App) -> usize {
    visible_items(app).len()
}

/// Find the cursor position that corresponds to a palette index.
/// Returns 0 if the palette isn't found in the visible list.
fn palette_idx_to_cursor(app: &App, palette_idx: usize) -> usize {
    visible_items(app)
        .iter()
        .position(|item| matches!(item, PaletteSelectItem::Palette(pi) if *pi == palette_idx))
        .unwrap_or(0)
}

fn handle_palette_select(app: &mut App, code: KeyCode) -> bool {
    let len = palette_select_len(app);
    match code {
        KeyCode::Esc | KeyCode::Tab => {
            app.palette.preview_idx = None;
            app.palette.show_hidden = false;
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
            app.input.new_palette_dir = Some(app.cursor_dir());
            app.palette.preview_idx = None;
            app.input.buf.clear();
            app.input.mode = Some(InputMode::ItemName);
        }
        KeyCode::Char('a') => {
            // Add a new directory
            app.input.buf.clear();
            app.input.mode = Some(InputMode::AddDir);
        }
        KeyCode::Char('D') => {
            if let PaletteSelectItem::Palette(idx) = palette_select_item(app, app.palette.cursor) {
                let pf = &app.palette.palettes[idx];
                let palette_name = pf.name.clone();
                let colour_count = pf.colours.len();
                app.input.buf.clear();
                app.input.mode = Some(InputMode::YesOrNo {
                    prompt: format!("delete {palette_name} ({colour_count} colours)? [y/n]: "),
                    action: YesOrNoAction::DeletePalette { palette_idx: idx },
                });
            }
        }
        KeyCode::Char('f') => {
            // Set dir_formats for the highlighted directory
            let dir = app.cursor_dir();
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
        KeyCode::Char('H') => {
            // Toggle hide on the directory of the item under the cursor
            let dir = app.cursor_dir();
            if let Some(dg) = app.palette.dir_groups.iter_mut().find(|dg| dg.path == dir) {
                if !dg.hideable {
                    app.set_status_warn("cannot hide this directory");
                } else {
                    dg.hidden = !dg.hidden;
                    if dg.hidden {
                        app.hidden_dirs.insert(dir);
                    } else {
                        app.hidden_dirs.remove(&dir);
                    }
                    palette::save_hidden_dirs(&app.hidden_dirs);
                    app.clamp_palette_cursor();
                }
            }
        }
        KeyCode::Char('h') => {
            // Toggle global visibility of hidden dirs
            app.palette.show_hidden = !app.palette.show_hidden;
            app.clamp_palette_cursor();
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
        app.input.mode = None;
        app.input.buf.clear();
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
                    YesOrNoAction::DeleteColour {
                        colour_name,
                        palette_idx,
                    } => {
                        let colour_name = colour_name.clone();
                        let palette_idx = *palette_idx;
                        let mut deleted = false;
                        if let Some(pf) = app.palette.palettes.get_mut(palette_idx) {
                            if let Err(e) = pf.remove_colour(&colour_name) {
                                app.set_status_error(&e);
                            } else {
                                deleted = true;
                            }
                        }
                        app.rescan();
                        app.load_palette(app.palette.idx);
                        if deleted {
                            app.set_status_ok(&format!("deleted {colour_name}"));
                        }
                        app.mode = Mode::Preview;
                    }
                    YesOrNoAction::DeletePalette { palette_idx } => {
                        let palette_idx = *palette_idx;
                        let mut deleted_name = None;
                        if let Some(pf) = app.palette.palettes.get(palette_idx) {
                            let path = pf.path.clone();
                            if let Err(e) = palette::remove_palette_file(&path) {
                                app.set_status_error(&e);
                            } else {
                                deleted_name = Some(pf.name.clone());
                            }
                        }
                        app.rescan();
                        app.load_palette(app.palette.idx);
                        app.clamp_palette_cursor();
                        if let Some(name) = deleted_name {
                            app.set_status_ok(&format!("deleted {name}"));
                        }
                        app.mode = Mode::PaletteSelect;
                    }
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
