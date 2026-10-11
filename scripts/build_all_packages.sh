#!/usr/bin/env bash
set -e

# ==============================================================================
# Vizo Image Viewer - Multi-Distribution Packaging Script
# Generates native installation packages:
#   - Ubuntu / Debian / Mint (.deb)
#   - Arch Linux / Manjaro (.pkg.tar.zst)
#   - Fedora / RHEL / CentOS (.rpm)
#   - Universal Standalone Linux Tarball (.tar.gz)
#   - Portable Windows ZIP (.zip)
# ==============================================================================

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

VERSION=$(grep '^version = ' Cargo.toml | head -n1 | cut -d '"' -f2)
PKGREL="1"

DIST_DIR="$PROJECT_DIR/dist"
mkdir -p "$DIST_DIR"

echo "============================================="
echo "🔨 Building Vizo Packages for All Platforms (v$VERSION-$PKGREL)"
echo "============================================="

# 1. Build Linux release binary
echo "⚙ [1/5] Building Linux release binary..."
cargo build --release

# 2. Package Ubuntu / Debian (.deb)
echo "📦 [2/5] Creating Ubuntu / Debian (.deb) package..."
DEB_BUILD="/tmp/vizo_deb_build"
rm -rf "$DEB_BUILD"
mkdir -p "$DEB_BUILD/debian" \
         "$DEB_BUILD/data/usr/bin" \
         "$DEB_BUILD/data/usr/share/applications" \
         "$DEB_BUILD/data/usr/share/icons/hicolor/scalable/apps" \
         "$DEB_BUILD/data/usr/share/doc/vizo"

cp target/release/vizo "$DEB_BUILD/data/usr/bin/vizo"
chmod 755 "$DEB_BUILD/data/usr/bin/vizo"
cp vizo.desktop "$DEB_BUILD/data/usr/share/applications/vizo.desktop"
cp assets/vizo.svg "$DEB_BUILD/data/usr/share/icons/hicolor/scalable/apps/vizo.svg"
cp README.md "$DEB_BUILD/data/usr/share/doc/vizo/README.md"

cat << EOF > "$DEB_BUILD/control"
Package: vizo
Version: ${VERSION}-${PKGREL}
Section: graphics
Priority: optional
Architecture: amd64
Maintainer: Altay Kireççi
Installed-Size: 24000
Homepage: https://github.com/altaykirecci/vizo
Description: High performance image and camera RAW viewer
 Vizo is a modern image and Camera RAW viewer written in Rust.
 Supports NEF, CR2, SR2, ARW, DNG, JPEG, PNG, WebP, TIFF, GIF, BMP, etc.
EOF

cat << 'EOF' > "$DEB_BUILD/postinst"
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true
fi
exit 0
EOF
chmod 755 "$DEB_BUILD/postinst"

echo "2.0" > "$DEB_BUILD/debian-binary"
tar -czf "$DEB_BUILD/control.tar.gz" --owner=0 --group=0 -C "$DEB_BUILD" control postinst
tar -czf "$DEB_BUILD/data.tar.gz" --owner=0 --group=0 -C "$DEB_BUILD/data" usr
ar rc "$DIST_DIR/vizo_${VERSION}-${PKGREL}_amd64.deb" "$DEB_BUILD/debian-binary" "$DEB_BUILD/control.tar.gz" "$DEB_BUILD/data.tar.gz"
rm -rf "$DEB_BUILD"
echo "   -> Created $DIST_DIR/vizo_${VERSION}-${PKGREL}_amd64.deb"

# 3. Package Arch Linux (.pkg.tar.zst)
echo "📦 [3/5] Creating Arch Linux (.pkg.tar.zst) package..."
if command -v makepkg >/dev/null 2>&1; then
    makepkg -f -R --nodeps
    mv vizo-*-x86_64.pkg.tar.zst "$DIST_DIR/"
    echo "   -> Created $DIST_DIR/vizo-${VERSION}-${PKGREL}-x86_64.pkg.tar.zst"
fi

# 4. Package Fedora / RHEL (.rpm)
echo "📦 [4/5] Creating Fedora (.rpm) package..."
if command -v cargo-generate-rpm >/dev/null 2>&1; then
    cargo-generate-rpm -o "$DIST_DIR/vizo-${VERSION}-${PKGREL}.x86_64.rpm"
    echo "   -> Created $DIST_DIR/vizo-${VERSION}-${PKGREL}.x86_64.rpm"
fi

# 5. Package Standalone Linux Tarball & Windows Zip
echo "📦 [5/5] Creating Standalone Tarball & Windows package..."
STANDALONE_DIR="/tmp/vizo_standalone"
rm -rf "$STANDALONE_DIR"
mkdir -p "$STANDALONE_DIR/vizo-linux-x86_64"
cp target/release/vizo \
   assets/vizo.svg \
   vizo.desktop \
   install.sh \
   uninstall.sh \
   README.md \
   "$STANDALONE_DIR/vizo-linux-x86_64/"

tar -czf "$DIST_DIR/vizo-linux-x86_64.tar.gz" -C "$STANDALONE_DIR" vizo-linux-x86_64
rm -rf "$STANDALONE_DIR"
echo "   -> Created $DIST_DIR/vizo-linux-x86_64.tar.gz"

if [ -f "target/x86_64-pc-windows-gnu/release/vizo.exe" ]; then
    python3 -c "import zipfile
with zipfile.ZipFile('$DIST_DIR/vizo-windows-x86_64.zip', 'w', zipfile.ZIP_DEFLATED) as z:
    z.write('target/x86_64-pc-windows-gnu/release/vizo.exe', 'vizo.exe')
    z.write('README.md', 'README.md')
"
    echo "   -> Created $DIST_DIR/vizo-windows-x86_64.zip"
fi

echo ""
echo "✅ All packages built successfully in $DIST_DIR:"
ls -lh "$DIST_DIR"
