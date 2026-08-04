# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

Come Back Soon...

## [Releases]

- [1.1.0](https://github.com/Khitboksy/palette-tui/releases/tag/v1.1.0)
- [1.0.1](https://github.com/Khitboksy/palette-tui/releases/tag/v1.0.1)
- [1.0.0](https://github.com/Khitboksy/palette-tui/releases/tag/v1.0.0)

## [v1.1.0] -- 2026-08-04

### Added

**Config option**(*s*):

- `DEV_OPTIONS` env var gates dev-only features, like scanning `./palettes` if you clone the repo.
- `hidden.json` lives inside `~/config/palette`, and is a persistent working copy of any hidden directories.

  > `hidden.json` is not a symlink by default for any platform. this is to prevent read-only issues.

**CLI**:

- `--version` / `-v` flag prints the application version and exits.

**UI**:

- Persistent version display in the bottom right corner.

**Dev Tools**:

- `bump-version` script for version management (`scripts/bump.rs`)

### Fixed

**Bug**(*s*):

- *Delete* and *overwrite* prompts now clear input state correctly after writing
- Status messages no longer linger when switching palettes or navigating colours
- `PaletteSelect` cursor now starts at the current palette instead of the top of the list
- H key now works on directories that contain palettes (*previously only empty dirs*)

### Changed

**Nix**:

- Nix HM module copies palettes instead of symlinking so they are writable

### Internal

**Refactor**:

- Extracted several helpers to reduce duplication (`reset_input`, `prev`/`next` cursor, `enter_palette_select`, similar-to rendering, palette select traversal)
- Unified palette loading between `App::new` and `load_palette`

## [v1.0.1] -- 2026-08-02

### Added

**Config option**(*s*):

- `[default].palette`: specify which palette to open on launch (

  ```toml
  [default]
  dir = "/path/to/old/default_dir"
  palette = "fileName"
  ```

)

- `default_palettes` directory (`~/.config/palette/palettes`) scanned automatically, hidden when empty

**Binds**

- `D` | Delete palettes and colours from within. (only in preview and palette select)
- `H` | Hide directories from the palette list, toggled per-directory
- `h` | Toggle visibility of hidden directories

### Changed

**Nix**:

- Nix modules updated to support the new default config options

### Internal

**Refactor**:

- Refactored `InputMode` into generic variants with an action enum
- Consolidated input handlers into generic functions
- Extracted helpers for clipboard, cursor clamping, hotkey hints, and pair mode
- Prompt rendering updated to match new `InputMode` variants

**Performance**:

- Borrow `InputMode` instead of cloning every frame
