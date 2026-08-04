mod app;
mod colour;
mod helpers;
mod input;
mod palette;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};

use app::{App, Mode};
use input::handle_key;
use palette::Config;

fn main() -> io::Result<()> {

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("palette-tui: v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut terminal = ratatui::init();

    let config = Config::load();
    let mut app = App::new(config);

    // Fallback: if no palettes found, try ./palettes relative to cwd
    // Only relevent when running via dev tooling.
    if app.palette.palettes.is_empty() {
        let fallback = std::path::Path::new("palettes");
        if fallback.is_dir() {
            let dirs = vec![fallback.to_path_buf()];
            let (palettes, dir_groups, scan_warnings) = palette::scan_directories(&dirs, &[]);
            app.palette.palettes = palettes;
            app.palette.dir_groups = dir_groups;
            for msg in scan_warnings {
                app.set_status_warn(&msg);
            }
            if !app.palette.palettes.is_empty() {
                app.load_palette(0);
            }
        }
    }

    loop {
        // Compute similar-to colours before rendering
        if app.dirty && !app.current_empty {
            app.compute_similar_to();
            app.dirty = false;
        }
        // Render
        terminal.draw(|frame| {
            ui::render_background(frame, &app.theme);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(frame.area());
            ui::render_list(frame, chunks[0], &app);
            if app.mode == Mode::PaletteSelect {
                ui::render_palette_select(frame, chunks[1], &app);
                if app.input.mode.is_some() {
                    let inner = helpers::inset_rect(chunks[1], 1);
                    ui::render_input_prompt(frame, inner, &app);
                }
            } else {
                ui::render_preview(frame, chunks[1], &app);
            }
        })?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && handle_key(key, &mut app)
        {
            break;
        }

        // Auto-clear expired status messages
        if let Some(expiry) = app.status.expiry
            && std::time::Instant::now() >= expiry
        {
            app.clear_status();
            // Re-prompt for directory if retry flag is set
            if app.input.add_dir_retry && app.mode == app::Mode::PaletteSelect {
                app.input.add_dir_retry = false;
                app.input.buf.clear();
                app.input.mode = Some(app::InputMode::AddDir);
            } else {
                app.input.add_dir_retry = false;
            }
        }
    }

    ratatui::restore();
    Ok(())
}
