#!/usr/bin/env bash
set -euo pipefail

REPO="Khitboksy/palette-tui"
BIN_DIR="${BIN_DIR:-/usr/local/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/palette"
PALETTES_DIR="$CONFIG_DIR/palettes"

# Detect platform
case "$(uname -s)" in
    Linux*)  OS=linux;;
    Darwin*) OS=darwin;;
    *)       echo "Error: unsupported OS $(uname -s)" >&2; exit 1;;
esac

case "$(uname -m)" in
    x86_64|amd64)   ARCH=x86_64;;
    aarch64|arm64)   ARCH=aarch64;;
    *)               echo "Error: unsupported architecture $(uname -m)" >&2; exit 1;;
esac

TARGET="${ARCH}-unknown-${OS}-gnu"
[ "$OS" = "darwin" ] && TARGET="${ARCH}-apple-darwin"

# Get latest release tag from GitHub API
TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
if [ -z "$TAG" ]; then
    echo "Error: could not find latest release" >&2
    exit 1
fi

ASSET="palette-${TAG}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading ${ASSET}..."
if ! curl -fSL -o "$TMPDIR/$ASSET" "$URL"; then
    echo "Error: failed to download $URL" >&2
    echo "Check that a release exists for your platform: https://github.com/${REPO}/releases" >&2
    exit 1
fi

echo "Extracting..."
tar xzf "$TMPDIR/$ASSET" -C "$TMPDIR"

EXTRACTED="$TMPDIR/palette-${TAG}-${TARGET}"

echo "Installing binary to $BIN_DIR..."
install -Dm755 "$EXTRACTED/palette" "$BIN_DIR/palette"

echo "Setting up config..."
mkdir -p "$PALETTES_DIR" "$CONFIG_DIR/themes"

copied=0
for f in "$EXTRACTED"/palettes/*.json; do
    [ -f "$f" ] || continue
    name="$(basename "$f")"
    if [ ! -f "$PALETTES_DIR/$name" ]; then
        install -Dm644 "$f" "$PALETTES_DIR/$name"
        copied=$((copied + 1))
    fi
done

echo "Done. palette installed to $BIN_DIR/palette"
if [ "$copied" -gt 0 ]; then
    echo "Copied $copied sample palette(s) to $PALETTES_DIR"
fi
echo "Run 'palette' to start."
