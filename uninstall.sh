#!/usr/bin/env bash
set -e

# ==============================================================================
# Vizo Image Viewer - Linux Uninstaller Script
# ==============================================================================

if [ "$1" == "--system" ] || [ "$EUID" -eq 0 ]; then
    echo "🗑 Removing Vizo system-wide (/usr/local)..."
    rm -f "/usr/local/bin/vizo"
    rm -f "/usr/share/applications/vizo.desktop"
    rm -f "/usr/share/icons/hicolor/scalable/apps/vizo.svg"
else
    echo "🗑 Removing Vizo for user $USER..."
    rm -f "$HOME/.local/bin/vizo"
    rm -f "$HOME/.local/share/applications/vizo.desktop"
    rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/vizo.svg"
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi

echo "✅ Vizo has been uninstalled."
