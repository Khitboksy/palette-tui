use ratatui::layout::Rect;
use std::path::Path;

// Collapse `$HOME` prefix in a path to `~` for display.
pub fn collapse_home(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    if let Some(rest) = path.strip_prefix(&home) {
        format!("~{}", rest)
    } else {
        path.to_string()
    }
}

// Resolve a relative path against `$HOME`. Absolute paths are returned as-is.
pub fn resolve_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix('~') {
        let home = std::env::var("HOME").unwrap_or_default();
        if rest.is_empty() {
            home
        } else {
            format!("{}{}", home, rest)
        }
    } else if Path::new(path).is_absolute() {
        path.to_string()
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/{}", home, path)
    }
}

// Check if a user's yes/no answer is a "yes" variant.
pub fn is_yes(answer: &str) -> bool {
    matches!(answer, "y" | "Y" | "yes" | "YES")
}

// Inset a `Rect` by `n` cells on all sides, using `saturating_sub` for safety.
pub fn inset_rect(area: Rect, n: u16) -> Rect {
    Rect {
        x: area.x + n,
        y: area.y + n,
        width: area.width.saturating_sub(2 * n),
        height: area.height.saturating_sub(2 * n),
    }
}

// Validate a name: alphanumeric, hyphens, underscores only.
pub fn validate_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

// Sanitize a name to only contain [a-zA-Z0-9_-].
// Strips everything else, collapses consecutive hyphens/underscores,
// and trims leading/trailing hyphens/underscores.
// Returns empty string if nothing valid remains.
pub fn sanitize_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    // Collapse consecutive hyphens/underscores into single hyphen
    let mut result = String::with_capacity(cleaned.len());
    let mut last_was_sep = false;
    for c in cleaned.chars() {
        if c == '-' || c == '_' {
            if !last_was_sep {
                result.push('-');
            }
            last_was_sep = true;
        } else {
            result.push(c);
            last_was_sep = false;
        }
    }
    // Trim leading/trailing hyphens
    result.trim_matches('-').to_string()
}

// Validate a directory path input
// alphanumerics, `/`, `.`, `-`, `_`, `~`.
pub fn validate_dir_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let last = path.as_bytes()[path.len() - 1];
    if !last.is_ascii_alphanumeric() && last != b'/' && last != b' ' {
        return false;
    }
    path.bytes().all(|b| {
        b.is_ascii_alphanumeric() || b == b'/' || b == b'.' || b == b'-' || b == b'_' || b == b'~'
    })
}
