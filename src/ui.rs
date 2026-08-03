use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::{App, InputMode, Mode, ThemeColors};
use crate::colour::*;
use crate::helpers;

// Fill the entire frame with the theme background colour.
// If bg is Color::Reset, the terminal's default background is used.
pub fn render_background(frame: &mut Frame, theme: &ThemeColors) {
    if theme.bg == Color::Reset {
        return;
    }
    let area = frame.area();
    let bg_style = Style::default().bg(theme.bg);
    let block = Block::default().style(bg_style);
    frame.render_widget(block, area);
}

fn hotkey_letter(letter: &str, theme: &ThemeColors) -> Span<'static> {
    Span::styled(letter.to_string(), Style::default().fg(theme.hotkey))
}

fn hotkey_sep(sep: &str, theme: &ThemeColors) -> Span<'static> {
    Span::styled(sep.to_string(), Style::default().fg(theme.hotkey_sep))
}

/// Build hotkey hint spans from a DSL string.
/// `|` and `-` become separators, everything else is a hotkey letter.
fn hotkey_spans(dsl: &str, theme: &ThemeColors) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = dsl.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '|' | '-' => {
                spans.push(hotkey_sep(&chars[i].to_string(), theme));
                i += 1;
            }
            ' ' => {
                // Leading space on first letter, or trailing space on last letter
                if spans.is_empty() {
                    // Leading space
                    if i + 1 < chars.len() && chars[i + 1] != '|' && chars[i + 1] != '-' {
                        spans.push(hotkey_letter(&format!(" {}", chars[i + 1]), theme));
                        i += 2;
                    } else {
                        i += 1;
                    }
                } else {
                    // Trailing space
                    if let Some(last) = spans.last_mut() {
                        *last = Span::styled(
                            format!("{} ", last.content),
                            Style::default().fg(theme.hotkey),
                        );
                    }
                    i += 1;
                }
            }
            c => {
                spans.push(hotkey_letter(&c.to_string(), theme));
                i += 1;
            }
        }
    }
    spans
}

// render_list. left panel colour list with swatch preview
pub fn render_list(frame: &mut Frame, area: Rect, app: &App) {
    // Get current palette path
    let current_path = if let Some(preview_idx) = app.palette.preview_idx {
        if preview_idx < app.palette.palettes.len() {
            Some(
                app.palette.palettes[preview_idx]
                    .source_dir
                    .display()
                    .to_string(),
            )
        } else {
            None
        }
    } else if !app.palette.palettes.is_empty() && app.palette.idx < app.palette.palettes.len() {
        Some(
            app.palette.palettes[app.palette.idx]
                .source_dir
                .display()
                .to_string(),
        )
    } else {
        None
    };

    // Borrow colour list items and draw the box
    let items: Vec<ListItem> = {
        let colour_iter: Box<dyn Iterator<Item = (usize, &str, &str)>> =
            if let Some(preview_idx) = app.palette.preview_idx {
                if let Some(pf) = app.palette.palettes.get(preview_idx) {
                    Box::new(
                        pf.colours
                            .iter()
                            .enumerate()
                            .map(|(i, ce)| (i, ce.name.as_str(), ce.hex.as_str())),
                    )
                } else {
                    Box::new(
                        app.colours
                            .iter()
                            .enumerate()
                            .map(|(i, (n, h))| (i, n.as_str(), h.as_str())),
                    )
                }
            } else {
                Box::new(
                    app.colours
                        .iter()
                        .enumerate()
                        .map(|(i, (n, h))| (i, n.as_str(), h.as_str())),
                )
            };

        colour_iter
            .map(|(i, name, hex)| {
                let is_empty = hex.trim_start_matches('#').is_empty();
                let c = hex_to_color(hex);
                let fg = to_ratatui_color(&c);
                let text_fg = textcolor(&c);

                let is_selected = i == app.selected;
                let is_paired = app.pair.idx == Some(i);
                let is_pair_cursor = app.mode == Mode::PairSelect && i == app.pair.cursor;

                let prefix = if is_pair_cursor || is_paired {
                    Span::styled("> ", Style::default().fg(app.theme.pointer_paired))
                } else if is_selected {
                    Span::styled("> ", Style::default().fg(app.theme.pointer))
                } else {
                    Span::raw("  ")
                };

                let col1_style = if is_empty {
                    Style::default().fg(app.theme.empty)
                } else if is_selected || is_paired {
                    Style::default().fg(fg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(fg)
                };

                let col2 = if is_empty {
                    Span::styled(
                        format!("{:<14}", "(empty)"),
                        Style::default().fg(app.theme.empty),
                    )
                } else {
                    Span::styled(format!("{name:<14}"), Style::default().fg(text_fg).bg(fg))
                };

                ListItem::new(Line::from(vec![
                    prefix,
                    Span::styled(format!("{name:<14}"), col1_style),
                    Span::raw("  "),
                    col2,
                ]))
            })
            .collect()
    };

    let title = if let Some(preview_idx) = app.palette.preview_idx {
        if preview_idx < app.palette.palettes.len() {
            format!(" {} ", app.palette.palettes[preview_idx].name)
        } else {
            " palette ".to_string()
        }
    } else if !app.palette.palettes.is_empty() && app.palette.idx < app.palette.palettes.len() {
        format!(" {} ", app.palette.palettes[app.palette.idx].name)
    } else {
        " palette ".to_string()
    };

    // Left panel is focused in Preview/PairSelect, unfocused otherwise
    let border_colour = if app.mode == Mode::Preview || app.mode == Mode::PairSelect {
        app.theme.border_focus
    } else {
        app.theme.border_unfocus
    };

    let mut block = Block::default()
        .title(title)
        .title_style(Style::default().fg(app.theme.fg))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_colour));

    // Add path to bottom-left of border (collapse $HOME to ~)
    if let Some(path) = &current_path {
        let display_path = helpers::collapse_home(path);
        block = block.title_bottom(Line::from(Span::styled(
            format!(" {display_path} "),
            Style::default().fg(app.theme.path),
        )));
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Scroll offset: keep the selected item visible
    let visible_height = inner.height as usize;
    let total_items = items.len();
    let scroll_offset = if total_items > visible_height && app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    // Only render visible items
    let visible_items: Vec<ListItem> = items
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();
    let list = List::new(visible_items);
    frame.render_widget(list, inner);

    // Keybind hint on top-right of list border -- only in preview mode
    if app.mode == Mode::Preview {
        let hint_spans = hotkey_spans(" tab|h-j-k-l|r-D ", &app.theme);
        let hint_width = 19u16;
        frame.render_widget(
            Paragraph::new(Line::from(hint_spans)),
            Rect {
                x: area.x + area.width - 18,
                y: area.y,
                width: hint_width,
                height: 1,
            },
        );
    }
}

// render_swatch_border. The checkerboard border around a colour swatch
pub fn render_swatch_border(
    frame: &mut Frame,
    area: Rect,
    colour: Color,
    theme: &crate::app::ThemeColors,
) {
    let light_bg = theme.swatch_light;
    let dark_bg = theme.swatch_dark;

    for y in 0..area.height {
        let row_area = Rect {
            x: area.x,
            y: area.y + y,
            width: area.width,
            height: 1,
        };
        if y == 0 || y == area.height - 1 {
            // Top/bottom gets pairs of 2 chars alternating checkerboard
            let mut spans = Vec::new();
            let mut x = 0u16;
            while x < area.width {
                let pair_idx = x / 2;
                let bg = if (pair_idx + y).is_multiple_of(2) {
                    light_bg
                } else {
                    dark_bg
                };
                let pair_len = std::cmp::min(2, area.width - x);
                spans.push(Span::styled(
                    " ".repeat(pair_len as usize),
                    Style::default().bg(bg),
                ));
                x += pair_len;
            }
            frame.render_widget(Paragraph::new(Line::from(spans)), row_area);
        } else {
            // Sides get 2 chars on left, colour in middle, 2 chars on right
            let left_bg = if y % 2 == 0 { light_bg } else { dark_bg };
            let right_bg = if y % 2 == 0 { dark_bg } else { light_bg };
            let inner_width = (area.width as usize).saturating_sub(4);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("  ", Style::default().bg(left_bg)),
                    Span::styled(" ".repeat(inner_width), Style::default().bg(colour)),
                    Span::styled("  ", Style::default().bg(right_bg)),
                ])),
                row_area,
            );
        }
    }
}

// render_input_prompt. standalone input prompt overlay
pub fn render_input_prompt(frame: &mut Frame, inner: Rect, app: &App) {
    if let Some(ref input_mode) = app.input.mode {
        // Toggle UI
        if let InputMode::Toggles { hex, hsl, rgb, .. } = input_mode {
            let focused = app.input.format_focused;
            let formats = [("hex", *hex), ("hsl", *hsl), ("rgb", *rgb)];
            let mut spans = vec![Span::styled(
                "formats: ",
                Style::default()
                    .fg(app.theme.input_fg)
                    .bg(app.theme.input_bg),
            )];
            for (i, (name, on)) in formats.iter().enumerate() {
                let marker = if *on { "x" } else { " " };
                let label = format!("[{marker}] {name}");
                let style = if i == focused {
                    Style::default()
                        .fg(app.theme.input_text)
                        .bg(app.theme.input_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(app.theme.input_fg)
                        .bg(app.theme.input_bg)
                };
                spans.push(Span::styled(label, style));
                if i < formats.len() - 1 {
                    spans.push(Span::styled("  ", Style::default().bg(app.theme.input_bg)));
                }
            }
            let input_line = Line::from(spans);
            let input_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(1),
                width: inner.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(input_line), input_area);
            return;
        }

        // Text input prompts
        let buf = &app.input.buf;
        let prompt_str = match input_mode {
            InputMode::HexInput => String::from("hex: "),
            InputMode::YesOrName { .. } => {
                // Extract name from mode for the prompt display
                if let InputMode::YesOrName { name } = input_mode {
                    format!("Overwrite '{name}'? [y/Name]: ")
                } else {
                    unreachable!()
                }
            }
            InputMode::ItemName => String::from("Name: "),
            InputMode::YesOrNo { prompt, .. } => prompt.clone(),
            InputMode::NewPaletteHex => String::from("initial hex: "),
            InputMode::AddDir => String::from("directory path: "),
            InputMode::Toggles { .. } => unreachable!(),
        };
        let input_line = Line::from(vec![
            Span::styled(
                &prompt_str,
                Style::default()
                    .fg(app.theme.input_fg)
                    .bg(app.theme.input_bg),
            ),
            Span::styled(
                buf.as_str(),
                Style::default()
                    .fg(app.theme.input_text)
                    .bg(app.theme.input_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "_",
                Style::default()
                    .fg(app.theme.input_fg)
                    .bg(app.theme.input_bg),
            ),
        ]);
        let input_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        frame.render_widget(Paragraph::new(input_line), input_area);
    }
}

// render_preview. right panel: swatch, info, sample text, menus
// Handles Preview, Command, Edit, PairSelect modes

pub fn render_preview(frame: &mut Frame, area: Rect, app: &App) {
    let colour = &app.current;
    let hex = app.current_hex();
    let ratatui_colour = to_ratatui_color(colour);
    let text_fg = textcolor(colour);

    // Border around the preview panel -- colour name as title, top-left, white
    let colour_name = if app.selected == 0 && app.is_random {
        "custom".to_string()
    } else if app.selected < app.visible_len() {
        let (name, _) = app.visible_entry(app.selected);
        name.to_string()
    } else {
        "custom".to_string()
    };
    let block_title = format!(" {colour_name} ");

    // Right panel is focused in Command/Edit, unfocused in Preview/PairSelect
    let border_colour = if app.mode == Mode::Command || app.mode == Mode::Edit {
        app.theme.border_focus
    } else {
        app.theme.border_unfocus
    };

    let block = Block::default()
        .title(Span::styled(block_title, Style::default().fg(app.theme.fg)))
        .title_style(Style::default().fg(app.theme.fg))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_colour));
    frame.render_widget(block, area);

    // Inner area (inside the border)
    let inner = helpers::inset_rect(area, 1);

    // Swatch area (including checkerboard border)
    // Internal fill: (width-4) x (height-2) cells, appears square with 2:1 aspect
    let swatch_w: u16 = 20;
    let swatch_h: u16 = 10;
    let swatch_area = Rect {
        x: inner.x + 2,
        y: inner.y + 1,
        width: swatch_w,
        height: swatch_h,
    };

    if app.current_empty {
        // Render checkerboard border with no fill (use theme bg)
        render_swatch_border(frame, swatch_area, app.theme.bg, &app.theme);
    } else {
        // Render the checkerboard border and colour fill
        render_swatch_border(frame, swatch_area, ratatui_colour, &app.theme);
    }

    // Info to the right of swatch, aligned with swatch top (command/edit mode)
    if app.mode != Mode::Preview {
        let info_x = swatch_area.x + swatch_area.width + 1;
        let info_y = swatch_area.y + 1;
        let info_width = inner.width.saturating_sub(info_x - inner.x);

        if app.current_empty {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "(empty)",
                    Style::default().fg(app.theme.empty),
                ))),
                Rect {
                    x: info_x,
                    y: info_y,
                    width: info_width,
                    height: 1,
                },
            );
        } else {
            // Hex / RGB / HSL lines (3 lines)
            let info_lines = vec![
                Line::from(vec![
                    Span::styled("Hex: ", Style::default().fg(app.theme.fg)),
                    Span::styled(
                        hex.to_uppercase(),
                        Style::default()
                            .fg(ratatui_colour)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("RGB: ", Style::default().fg(app.theme.fg)),
                    Span::styled(format_rgb(colour), Style::default().fg(app.theme.fg)),
                ]),
                Line::from(vec![
                    Span::styled("HSL: ", Style::default().fg(app.theme.fg)),
                    Span::styled(format_hsl(colour), Style::default().fg(app.theme.fg)),
                ]),
            ];
            frame.render_widget(
                Paragraph::new(info_lines),
                Rect {
                    x: info_x,
                    y: info_y,
                    width: info_width,
                    height: 3,
                },
            );

            // Similar colours (pre-computed in main loop)
            let sim_y = info_y + 4; // skip a blank line

            if !app.similar_to.is_empty() {
                let mut sim_lines: Vec<Line> = Vec::new();
                sim_lines.push(Line::from(Span::styled(
                    "similar to:",
                    Style::default().fg(app.theme.hint),
                )));

                // Render up to 3 rows, each with column 1 (named) and column 2 (palette)
                // Entries 0-2 are named colours (keys 1-3), entries 3-5 are palette (keys 4-6)
                for row in 0..3 {
                    let mut spans: Vec<Span> = Vec::new();
                    spans.push(Span::raw("  "));

                    // Column 1: named colour (index row)
                    if let Some((name, hex_val)) = app.similar_to.get(row) {
                        let c = hex_to_color(hex_val);
                        let r = to_ratatui_color(&c);
                        let fg = textcolor(&c);
                        // Show > for paired item (match by name, not index)
                        if app.pair.similar_name.as_deref() == Some(name.as_str()) {
                            spans.push(Span::styled(
                                " > ",
                                Style::default().fg(app.theme.pointer_paired),
                            ));
                        } else {
                            spans.push(Span::styled(
                                format!("[{}]", row + 1),
                                Style::default().fg(app.theme.hint),
                            ));
                        }
                        spans.push(Span::styled(
                            format!("{name:<14}"),
                            Style::default().fg(fg).bg(r),
                        ));
                    } else {
                        spans.push(Span::raw("                   "));
                    }

                    spans.push(Span::raw("  "));

                    // Column 2: palette colour (index row + 3)
                    if let Some((name, hex_val)) = app.similar_to.get(row + 3) {
                        let c = hex_to_color(hex_val);
                        let r = to_ratatui_color(&c);
                        let fg = textcolor(&c);
                        // Show > for paired item (match by name, not index)
                        if app.pair.similar_name.as_deref() == Some(name.as_str()) {
                            spans.push(Span::styled(
                                " > ",
                                Style::default().fg(app.theme.pointer_paired),
                            ));
                        } else {
                            spans.push(Span::styled(
                                format!("[{}]", row + 4),
                                Style::default().fg(app.theme.hint),
                            ));
                        }
                        spans.push(Span::styled(
                            format!("{name:<14}"),
                            Style::default().fg(fg).bg(r),
                        ));
                    }

                    sim_lines.push(Line::from(spans));
                }

                let height = sim_lines.len() as u16;
                frame.render_widget(
                    Paragraph::new(sim_lines),
                    Rect {
                        x: info_x,
                        y: sim_y,
                        width: info_width,
                        height,
                    },
                );
            }
        }
    }

    // Sample text below swatch
    let sample_y = swatch_area.y + swatch_area.height + 1;
    let sample_lines = vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled("extended example text", Style::default().fg(ratatui_colour)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "extended example text",
                Style::default().fg(text_fg).bg(ratatui_colour),
            ),
        ]),
    ];
    let sample_area = Rect {
        x: inner.x,
        y: sample_y,
        width: inner.width,
        height: 2,
    };
    frame.render_widget(Paragraph::new(sample_lines), sample_area);

    // Pair contrast samples
    let mut next_y = sample_y + 3;
    if let Some(ref paired) = app.pair.paired {
        let pair_hex = color_to_hex(paired);
        let pair_ratatui = to_ratatui_color(paired);

        let pair_lines = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("this text on {pair_hex} background"),
                    Style::default().fg(ratatui_colour).bg(pair_ratatui),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{pair_hex} text on this background"),
                    Style::default().fg(pair_ratatui).bg(ratatui_colour),
                ),
            ]),
        ];
        let pair_area = Rect {
            x: inner.x,
            y: next_y,
            width: inner.width,
            height: 2,
        };
        frame.render_widget(Paragraph::new(pair_lines), pair_area);
        next_y += 3;

        // Persistent paired status
        let paired_status = Line::from(vec![
            Span::styled("  paired: ", Style::default().fg(app.theme.action)),
            Span::styled(
                &app.pair.name,
                Style::default()
                    .fg(pair_ratatui)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        let paired_area = Rect {
            x: inner.x,
            y: next_y,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(Paragraph::new(paired_status), paired_area);
        next_y += 1;
    }

    // Status message
    if !app.status.msg.is_empty() {
        let y = if app.input.mode.is_some() {
            // When input prompt is active, status goes above it
            inner.y + inner.height.saturating_sub(2)
        } else {
            next_y
        };
        let status_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                format!("  {}", app.status.msg),
                Style::default().fg(app.status_color()),
            )])),
            status_area,
        );
        if app.input.mode.is_none() {
            next_y += 1;
        }
    }

    // Editing indicator in edit mode
    if app.mode == Mode::Edit {
        let name = app.edit.base_name.as_deref().unwrap_or("custom");
        let edit_colour = to_ratatui_color(&app.current);
        let edit_area = Rect {
            x: inner.x,
            y: next_y,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  editing: ", Style::default().fg(app.theme.action)),
                Span::styled(
                    name,
                    Style::default()
                        .fg(edit_colour)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            edit_area,
        );
        next_y += 1;
    }

    // Input prompt overlay (renders at bottom of panel)
    render_input_prompt(frame, inner, app);

    // Menus in command/edit/pairselect/paletteselect mode
    let mut menu_lines: Vec<Line> = Vec::new();
    if app.mode == Mode::Command || app.mode == Mode::Edit {
        menu_lines.push(Line::from(""));
    }
    if app.mode == Mode::Command {
        menu_lines.push(Line::from(vec![Span::styled(
            "  [e]dit  [i]nput  f[z]f",
            Style::default().fg(app.theme.fg),
        )]));
        menu_lines.push(Line::from(vec![Span::styled(
            "  [p]air  [P]air-clear",
            Style::default().fg(app.theme.fg),
        )]));
        menu_lines.push(Line::from(vec![Span::styled(
            "  [s]ex   [c]rgb  [d]hsl  [f]lip",
            Style::default().fg(app.theme.fg),
        )]));
    }
    if app.mode == Mode::Edit {
        menu_lines.push(Line::from(vec![Span::styled(
            "  [p]air  [P]air-clear",
            Style::default().fg(app.theme.fg),
        )]));
        menu_lines.push(Line::from(vec![Span::styled(
            "  [w]rite [e]xit f[z]f",
            Style::default().fg(app.theme.fg),
        )]));
        menu_lines.push(Line::from(vec![Span::styled(
            "  +[r]ed -[R]ed  +[g]reen -[G]reen  +[b]lue -[B]lue",
            Style::default().fg(app.theme.fg),
        )]));
        menu_lines.push(Line::from(vec![Span::styled(
            "  +[j]ue -[J]ue  +[l]ight -[L]ight  +sa[k]urate -sa[K]urate",
            Style::default().fg(app.theme.fg),
        )]));
    }
    if app.mode == Mode::PairSelect {
        menu_lines.push(Line::from(vec![Span::styled(
            "  choose pair colour: j/k to move, Enter to confirm, Esc to cancel",
            Style::default().fg(app.theme.hint),
        )]));
    }
    if !menu_lines.is_empty() {
        let menu_area = Rect {
            x: inner.x,
            y: next_y,
            width: inner.width,
            height: menu_lines.len() as u16,
        };
        frame.render_widget(Paragraph::new(menu_lines), menu_area);
    }

    // Keybind hints on bottom-left of preview border -- edit mode
    if app.mode == Mode::Edit {
        let hint_spans = hotkey_spans(" r-g-b|j-l-k|w-e-z ", &app.theme);
        let hint_width = 21u16;
        frame.render_widget(
            Paragraph::new(Line::from(hint_spans)),
            Rect {
                x: area.x + 1,
                y: area.y + area.height - 1,
                width: hint_width,
                height: 1,
            },
        );
    }
}

// render_palette_select. palette chooser overlay in the right panel
pub fn render_palette_select(frame: &mut Frame, area: Rect, app: &App) {
    let header_colour = app.theme.path;
    let empty_colour = app.theme.empty;

    // Build visual items: directory headers + palette entries
    // palette_cursor maps to palette indices (not visual lines)
    let mut items: Vec<ListItem> = Vec::new();
    let mut selectable_count: usize = 0;

    for dg in &app.palette.dir_groups {
        // Skip hidden dirs (hidden_when_empty || hidden) when empty and show_hidden is off
        if (dg.hidden_when_empty || dg.hidden)
            && dg.palette_indices.is_empty()
            && !app.palette.show_hidden
        {
            continue;
        }

        // Directory header (non-selectable)
        // Show path relative to home directory
        let dir_display = helpers::collapse_home(&dg.path.display().to_string());
        let header_span = Span::styled(
            format!("  {dir_display}"),
            Style::default()
                .fg(header_colour)
                .add_modifier(Modifier::BOLD),
        );
        items.push(ListItem::new(Line::from(header_span)));

        if dg.palette_indices.is_empty() {
            // Empty directory marker (selectable)
            let is_selected = selectable_count == app.palette.cursor;
            let prefix = if is_selected {
                Span::styled("  > ", Style::default().fg(app.theme.pointer))
            } else {
                Span::raw("    ")
            };
            let empty_span = Span::styled("(empty)", Style::default().fg(empty_colour));
            items.push(ListItem::new(Line::from(vec![prefix, empty_span])));
            selectable_count += 1;
        } else {
            // Palette entries
            for &pi in &dg.palette_indices {
                let pf = &app.palette.palettes[pi];
                let is_selected = selectable_count == app.palette.cursor;
                let display = format!("{} ({} colours)", pf.name, pf.colours.len());

                let prefix = if is_selected {
                    Span::styled("  > ", Style::default().fg(app.theme.pointer))
                } else {
                    Span::raw("    ")
                };

                let style = if is_selected {
                    Style::default()
                        .fg(app.theme.pointer)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                items.push(ListItem::new(Line::from(vec![
                    prefix,
                    Span::styled(display, style),
                ])));
                selectable_count += 1;
            }
        }
    }

    let block = Block::default()
        .title(Span::styled(
            " palettes (~) ",
            Style::default().fg(app.theme.fg),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focus));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);

    // Status message inside palette select (if not empty and in PaletteSelect mode)
    if !app.status.msg.is_empty() && app.mode == Mode::PaletteSelect {
        let y = if app.input.mode.is_some() {
            // When input prompt is active, status goes above it
            area.y + area.height - 3
        } else {
            area.y + area.height - 2
        };
        let status_area = Rect {
            x: area.x + 1,
            y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                &app.status.msg,
                Style::default().fg(app.status_color()),
            )])),
            status_area,
        );
    }

    // Keybind hint on bottom-left of palette select border
    let hint_spans = hotkey_spans(" h-H-D-|j-k|a-n-f ", &app.theme);
    let hint_width = 18u16;
    frame.render_widget(
        Paragraph::new(Line::from(hint_spans)),
        Rect {
            x: area.x + 1,
            y: area.y + area.height - 1,
            width: hint_width,
            height: 1,
        },
    );
}
