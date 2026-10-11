#!/usr/bin/env bash
set -e

# ==============================================================================
# Vizo Image Viewer - Linux Installer Script
# Installs binary, desktop launcher entry, and scalable icon.
# ==============================================================================

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_SOURCE="$PROJECT_DIR/target/release/vizo"

# Build release binary if not present
if [ ! -f "$BIN_SOURCE" ]; then
    echo "⚙ Release binary not found. Building with cargo --release..."
    if command -v cargo >/dev/null 2>&1; then
        cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"
    elif [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
        cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"
    else
        echo "❌ Error: cargo not found. Please install Rust or run 'cargo build --release' first."
        exit 1
    fi
fi

# Detect installation mode
if [ "$1" == "--system" ] || [ "$EUID" -eq 0 ]; then
    echo "📦 Installing Vizo system-wide (/usr/local)..."
    INSTALL_BIN="/usr/local/bin"
    INSTALL_APPS="/usr/share/applications"
    INSTALL_ICONS="/usr/share/icons/hicolor/scalable/apps"
else
    echo "📦 Installing Vizo for current user ($USER)..."
    INSTALL_BIN="$HOME/.local/bin"
    INSTALL_APPS="$HOME/.local/share/applications"
    INSTALL_ICONS="$HOME/.local/share/icons/hicolor/scalable/apps"
fi

mkdir -p "$INSTALL_BIN"
mkdir -p "$INSTALL_APPS"
mkdir -p "$INSTALL_ICONS"

# 1. Install executable
echo "-> Installing binary to $INSTALL_BIN/vizo..."
install -m 755 "$BIN_SOURCE" "$INSTALL_BIN/vizo"

# 2. Install icon
echo "-> Installing icon to $INSTALL_ICONS/vizo.svg..."
install -m 644 "$PROJECT_DIR/assets/vizo.svg" "$INSTALL_ICONS/vizo.svg"

# 3. Install desktop launcher
echo "-> Installing desktop entry to $INSTALL_APPS/vizo.desktop..."
install -m 644 "$PROJECT_DIR/vizo.desktop" "$INSTALL_APPS/vizo.desktop"
# Use absolute path for Exec so desktop runners and file managers with minimal PATH can find it
sed -i "s|^Exec=vizo|Exec=$INSTALL_BIN/vizo|" "$INSTALL_APPS/vizo.desktop"

# 4. Clean up any broken userapp definitions created by manual Open-With dialogues
rm -f "$INSTALL_APPS"/userapp-vizo-*.desktop 2>/dev/null || true

# 5. Update desktop & icon caches if tools exist
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$INSTALL_APPS" 2>/dev/null || true
fi

if [ -d "$HOME/.local/share/mime/packages" ] && command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database "$HOME/.local/share/mime" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$(dirname "$INSTALL_ICONS")" 2>/dev/null || true
fi

echo ""
echo "✅ Vizo Image Viewer successfully installed!"
echo "   Binary: $INSTALL_BIN/vizo"
echo "   Launcher: $INSTALL_APPS/vizo.desktop"
echo ""
echo "You can now run 'vizo' from terminal or open it from your application menu!"
