#!/usr/bin/env bash
# Regenerate the hicolor PNG app icons from the master in assets/app_icon.png.
# Run this whenever the master icon changes; commit the results under data/icons.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/assets/app_icon.png"
APP_ID="io.github.dosui"
SIZES=(16 32 48 64 128 256 512)

[ -f "$SRC" ] || { echo "missing master icon: $SRC" >&2; exit 1; }

# Resize $SRC to a square $1 px PNG at $2, using whatever tool is available.
resize() {
    local size="$1" out="$2"
    if command -v magick >/dev/null; then
        magick "$SRC" -resize "${size}x${size}" "$out"
    elif command -v convert >/dev/null; then
        convert "$SRC" -resize "${size}x${size}" "$out"
    elif command -v gdk-pixbuf-thumbnailer >/dev/null; then
        gdk-pixbuf-thumbnailer -s "$size" "$SRC" "$out"
    else
        echo "no image tool found (install ImageMagick or gdk-pixbuf)" >&2
        exit 1
    fi
}

for s in "${SIZES[@]}"; do
    dir="$ROOT/data/icons/hicolor/${s}x${s}/apps"
    mkdir -p "$dir"
    resize "$s" "$dir/$APP_ID.png"
    printf '  %sx%s\n' "$s" "$s"
done
echo "Generated hicolor icons under data/icons from $(basename "$SRC")."
