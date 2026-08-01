use pastel::Color;
use std::str::FromStr;

// Conversions: hex <-> pastel::Color

/// Parse a hex string (with or without '#') into a pastel::Color.
/// Falls back to black on malformed input.
pub fn hex_to_color(hex: &str) -> Color {
    let h = if hex.starts_with('#') {
        hex.to_string()
    } else {
        format!("#{hex}")
    };
    Color::from_str(&h).unwrap_or_else(|_| Color::black())
}

/// Format a pastel::Color as "#rrggbb" lowercase.
pub fn color_to_hex(c: &Color) -> String {
    c.to_rgb_hex_string(true)
}

/// Normalize a hex string: ensure '#'-prefixed, return None if invalid.
pub fn normalize_hex(input: &str) -> Option<String> {
    let hex = if input.starts_with('#') {
        input.to_string()
    } else {
        format!("#{input}")
    };
    if hex.len() == 7 && hex[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex)
    } else {
        None
    }
}

/// Format a colour as "hsl(hue, sat%, light%)" with consistent precision.
pub fn format_hsl(c: &Color) -> String {
    let hsla = c.to_hsla();
    format!(
        "hsl({:.0}, {:.1}%, {:.1}%)",
        hsla.h,
        hsla.s * 100.0,
        hsla.l * 100.0
    )
}

/// Format a colour as "rgb(r, g, b)".
pub fn format_rgb(c: &Color) -> String {
    let rgba = c.to_rgba();
    format!("rgb({}, {}, {})", rgba.r, rgba.g, rgba.b)
}

// Conversions: pastel::Color <-> ratatui::style::Color

/// Convert a pastel::Color to a ratatui Color::Rgb for terminal rendering.
pub fn to_ratatui_color(c: &Color) -> ratatui::style::Color {
    let rgba = c.to_rgba();
    ratatui::style::Color::Rgb(rgba.r, rgba.g, rgba.b)
}

// Text readability

/// Pick black or white for readable text on a given background colour.
/// Uses pastel's WCAG-compliant luminance calculation.
pub fn textcolor(c: &Color) -> ratatui::style::Color {
    let fg = c.text_color();
    to_ratatui_color(&fg)
}

// Colour transformations (delegated to pastel)

/// Lighten a colour by `amount` (0.0..1.0 fraction of lightness range).
pub fn lighten(c: &Color, amount: f64) -> Color {
    c.lighten(amount)
}
/// Darken a colour by `amount` (0.0..1.0 fraction of lightness range).
pub fn darken(c: &Color, amount: f64) -> Color {
    c.darken(amount)
}
/// Increase saturation by `amount` (0.0..1.0 fraction).
pub fn saturate(c: &Color, amount: f64) -> Color {
    c.saturate(amount)
}
/// Decrease saturation by `amount` (0.0..1.0 fraction).
pub fn desaturate(c: &Color, amount: f64) -> Color {
    c.desaturate(amount)
}
/// Rotate hue by `degrees` (positive = clockwise, wraps around 360).
pub fn rotate(c: &Color, degrees: f64) -> Color {
    c.rotate_hue(degrees)
}
/// Get the complementary colour (hue rotated by 180 degrees).
pub fn complement(c: &Color) -> Color {
    c.complementary()
}

// Named colour lookup

/// Find the N closest CSS/X11 named colours to a given colour, using
/// CIEDE2000 perceptual distance. Returns (name, Color) pairs.
pub fn closest_named_colors(c: &Color, n: usize) -> Vec<(&'static str, Color)> {
    let mut distances: Vec<(f64, usize)> = pastel::named::NAMED_COLORS
        .iter()
        .enumerate()
        .map(|(i, nc)| (c.distance_delta_e_ciede2000(&nc.color), i))
        .collect();
    distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    distances
        .iter()
        .take(n)
        .map(|(_, i)| {
            let nc = &pastel::named::NAMED_COLORS[*i];
            (nc.name, nc.color.clone())
        })
        .collect()
}

/// Generate a random hex colour string.
pub fn random_hex() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let r: u8 = rng.random();
    let g: u8 = rng.random();
    let b: u8 = rng.random();
    let c = Color::from_rgb(r, g, b);
    color_to_hex(&c)
}
