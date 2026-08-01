#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${BIN_DIR:-/usr/local/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/palette"

echo "Removing $BIN_DIR/palette..."
rm -f "$BIN_DIR/palette"

read -rp "Remove config directory $CONFIG_DIR? [y/N] " answer
if [[ "$answer" =~ ^[Yy]$ ]]; then
    rm -rf "$CONFIG_DIR"
    echo "Removed $CONFIG_DIR"
else
    echo "Kept $CONFIG_DIR"
fi

echo "Done."
