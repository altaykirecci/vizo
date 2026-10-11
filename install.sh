#!/usr/bin/env bash
set -e

# ==============================================================================
# Vizo Image Viewer - Linux Installer Script
# Installs binary, desktop launcher entry, and scalable icon.
# ==============================================================================

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 1. Locate pre-compiled executable (standalone package or project build)
if [ -f "$PROJECT_DIR/vizo" ]; then
    BIN_SOURCE="$PROJECT_DIR/vizo"
elif [ -f "$PROJECT_DIR/target/release/vizo" ]; then
    BIN_SOURCE="$PROJECT_DIR/target/release/vizo"
else
    echo "❌ Error: 'vizo' binary not found in $PROJECT_DIR."
    echo "   Please make sure the 'vizo' executable is in the same folder as install.sh."
    exit 1
fi

# 2. Locate SVG icon
if [ -f "$PROJECT_DIR/vizo.svg" ]; then
    ICON_SOURCE="$PROJECT_DIR/vizo.svg"
elif [ -f "$PROJECT_DIR/assets/vizo.svg" ]; then
    ICON_SOURCE="$PROJECT_DIR/assets/vizo.svg"
else
    echo "❌ Error: 'vizo.svg' icon not found."
    exit 1
fi

# 3. Locate desktop file
if [ -f "$PROJECT_DIR/vizo.desktop" ]; then
    DESKTOP_SOURCE="$PROJECT_DIR/vizo.desktop"
else
    echo "❌ Error: 'vizo.desktop' file not found."
    exit 1
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
install -m 644 "$ICON_SOURCE" "$INSTALL_ICONS/vizo.svg"

# 3. Install desktop launcher
echo "-> Installing desktop entry to $INSTALL_APPS/vizo.desktop..."
install -m 644 "$DESKTOP_SOURCE" "$INSTALL_APPS/vizo.desktop"
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
