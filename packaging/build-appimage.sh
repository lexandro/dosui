#!/usr/bin/env bash
# Build a self-contained dosui AppImage that bundles dosbox-staging + GTK4.
#
# The bundle deliberately ships the GTK *runtime* (via linuxdeploy-plugin-gtk)
# but NOT a GTK theme, and the plugin's forced GTK_THEME is neutralised, so the
# app follows the host theme (e.g. Mint-Y) on the target machine.
#
# Env overrides:
#   DOSBOX_STAGING_DIR      path to an extracted dosbox-staging (skips the search/download)
#   DOSBOX_STAGING_VERSION  version to download if none is found locally (default below)
#   DOSUI_TOOLS             cache dir for linuxdeploy tools + dosbox (default: ~/.local/opt)
set -euo pipefail

# Pinned dosbox-staging version, downloaded only when no local build is found.
DOSBOX_STAGING_VERSION="${DOSBOX_STAGING_VERSION:-0.82.2}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
APPDIR="$DIST/AppDir"
TOOLS="${DOSUI_TOOLS:-$HOME/.local/opt}"
export ARCH=x86_64
export APPIMAGE_EXTRACT_AND_RUN=1 # run the tool AppImages without FUSE

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }

# 1. Locate dosbox-staging (binary + resources), downloading a pinned build if
#    none is found. The directory holding the `dosbox` executable is what we need.
find_dosbox_dir() {
    local root="$1" bin
    [ -d "$root" ] || return 0
    bin="$(find "$root" -maxdepth 3 -type f -name dosbox -perm -u+x 2>/dev/null | sort | tail -1)"
    [ -n "$bin" ] && dirname "$bin"
}

if [ -n "${DOSBOX_STAGING_DIR:-}" ] && [ -x "$DOSBOX_STAGING_DIR/dosbox" ]; then
    DOSBOX_DIR="$DOSBOX_STAGING_DIR"
else
    DOSBOX_DIR="$(find_dosbox_dir "$HOME/.local/opt")"
fi

if [ -z "${DOSBOX_DIR:-}" ]; then
    ver="$DOSBOX_STAGING_VERSION"
    cache="$TOOLS/dosbox-staging-v$ver"
    DOSBOX_DIR="$(find_dosbox_dir "$cache")"
    if [ -z "$DOSBOX_DIR" ]; then
        log "downloading dosbox-staging v$ver"
        mkdir -p "$cache"
        tarball="dosbox-staging-linux-x86_64-v$ver.tar.xz"
        curl -fsSL -o "$cache/$tarball" \
            "https://github.com/dosbox-staging/dosbox-staging/releases/download/v$ver/$tarball"
        tar -xf "$cache/$tarball" -C "$cache"
        DOSBOX_DIR="$(find_dosbox_dir "$cache")"
    fi
fi

if [ -z "${DOSBOX_DIR:-}" ] || [ ! -x "$DOSBOX_DIR/dosbox" ]; then
    echo "dosbox-staging not found and the download failed." >&2
    echo "Set DOSBOX_STAGING_DIR to an extracted build, or check your network." >&2
    exit 1
fi
log "dosbox-staging: $DOSBOX_DIR"

# 2. Release build.
log "cargo build --release"
cargo build --release --manifest-path "$ROOT/Cargo.toml"

# 3. Assemble the AppDir.
log "assembling AppDir"
rm -rf "$APPDIR"
install -Dm755 "$ROOT/target/release/dosui" "$APPDIR/usr/bin/dosui"
install -Dm644 "$ROOT/data/io.github.dosui.desktop" \
    "$APPDIR/usr/share/applications/io.github.dosui.desktop"
for size in 16 32 48 64 128 256 512; do
    install -Dm644 "$ROOT/data/icons/hicolor/${size}x${size}/apps/io.github.dosui.png" \
        "$APPDIR/usr/share/icons/hicolor/${size}x${size}/apps/io.github.dosui.png"
done
# Bundle dosbox-staging next to dosui; the launcher finds it via $APPDIR/usr/bin/dosbox.
cp -a "$DOSBOX_DIR/dosbox" "$APPDIR/usr/bin/dosbox"
cp -a "$DOSBOX_DIR/resources" "$APPDIR/usr/bin/resources"

# 4. Fetch linuxdeploy + the GTK plugin (cached in $TOOLS).
mkdir -p "$TOOLS"
LD="$TOOLS/linuxdeploy-x86_64.AppImage"
LDGTK="$TOOLS/linuxdeploy-plugin-gtk.sh"
if [ ! -f "$LD" ]; then
    log "downloading linuxdeploy"
    curl -sSL -o "$LD" \
        "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
    chmod +x "$LD"
fi
if [ ! -f "$LDGTK" ]; then
    log "downloading linuxdeploy-plugin-gtk"
    curl -sSL -o "$LDGTK" \
        "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh"
    chmod +x "$LDGTK"
fi
# linuxdeploy discovers plugins on PATH as `linuxdeploy-plugin-<name>`.
ln -sf "$LDGTK" "$TOOLS/linuxdeploy-plugin-gtk"
export PATH="$TOOLS:$PATH"

# 5. Phase 1 — bundle GTK runtime + app deps into the AppDir (no packaging yet).
log "linuxdeploy: bundling GTK runtime"
"$LD" --appdir "$APPDIR" \
    --desktop-file "$APPDIR/usr/share/applications/io.github.dosui.desktop" \
    --icon-file "$APPDIR/usr/share/icons/hicolor/256x256/apps/io.github.dosui.png" \
    --plugin gtk

# 6. Follow the host theme: neutralise any forced GTK_THEME in the plugin hook.
for hook in "$APPDIR"/apprun-hooks/*gtk*.sh; do
    [ -f "$hook" ] || continue
    sed -i 's/^\s*export GTK_THEME=.*/# GTK_THEME left unset so the host theme wins (dosui)/' "$hook"
    log "patched $(basename "$hook") to keep the host GTK theme"
done

# 7. Phase 2 — package the AppDir into the final AppImage.
log "linuxdeploy: packaging AppImage"
( cd "$DIST" && "$LD" --appdir "$APPDIR" --output appimage )

log "done: $(ls "$DIST"/dosui*.AppImage 2>/dev/null | head -1)"
