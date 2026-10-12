#!/usr/bin/env bash
set -e

# ==============================================================================
# Vizo Image Viewer - macOS Packaging Script
# Generates native macOS distribution packages:
#   - macOS Application Bundle (Vizo.app)
#   - macOS Drag-and-Drop Disk Image (.dmg)
#   - Standalone Universal Archive (.tar.gz)
# ==============================================================================

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

if [ "$(uname)" != "Darwin" ]; then
    echo "⚠️ Warning: build_mac_package.sh is designed for macOS (Darwin)."
    echo "Current OS is $(uname). Native DMG and codesign tools may not be available."
fi

VERSION=$(grep '^version = ' Cargo.toml | head -n1 | cut -d '"' -f2)
ARCH=$(uname -m)

DIST_DIR="$PROJECT_DIR/dist"
mkdir -p "$DIST_DIR"

echo "=========================================================="
echo "🍏 Building Vizo macOS Distribution Packages (v$VERSION - $ARCH)"
echo "=========================================================="

# 1. Compile release binary
echo "⚙️  [1/4] Compiling release binary..."
cargo build --release

# 2. Ensure macOS icon (.icns) exists
echo "🎨 [2/4] Verifying macOS application icon (vizo.icns)..."
if [ ! -f "assets/vizo.icns" ] && [ -f "assets/vizo.svg" ] && command -v iconutil >/dev/null 2>&1; then
    echo "   -> Generating assets/vizo.icns from assets/vizo.svg..."
    python3 -c "
import os, subprocess, tempfile
with tempfile.TemporaryDirectory() as tmpdir:
    svg_path = os.path.abspath('assets/vizo.svg')
    out_icns = os.path.abspath('assets/vizo.icns')
    subprocess.run(['qlmanage', '-t', '-s', '1024', '-o', tmpdir, svg_path], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    base_png = os.path.join(tmpdir, 'vizo.svg.png')
    iconset = os.path.join(tmpdir, 'vizo.iconset')
    os.makedirs(iconset, exist_ok=True)
    sizes = [
        (16, '16x16'), (32, '16x16@2x'),
        (32, '32x32'), (64, '32x32@2x'),
        (128, '128x128'), (256, '128x128@2x'),
        (256, '256x256'), (512, '256x256@2x'),
        (512, '512x512'), (1024, '512x512@2x')
    ]
    for sz, name in sizes:
        subprocess.run(['sips', '-z', str(sz), str(sz), base_png, '--out', f'{iconset}/icon_{name}.png'], check=True, stdout=subprocess.DEVNULL)
    subprocess.run(['iconutil', '-c', 'icns', iconset, '-o', out_icns], check=True)
"
fi

# 3. Create macOS Application Bundle (Vizo.app)
echo "📦 [3/4] Creating Vizo.app bundle..."
APP_BUNDLE="$DIST_DIR/Vizo.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp target/release/vizo "$APP_BUNDLE/Contents/MacOS/vizo"
chmod 755 "$APP_BUNDLE/Contents/MacOS/vizo"

if [ -f "assets/vizo.icns" ]; then
    cp assets/vizo.icns "$APP_BUNDLE/Contents/Resources/vizo.icns"
fi

cat << 'EOF' > "$APP_BUNDLE/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Vizo</string>
    <key>CFBundleDisplayName</key>
    <string>Vizo Image Viewer</string>
    <key>CFBundleIdentifier</key>
    <string>com.altaykirecci.vizo</string>
    <key>CFBundleVersion</key>
    <string>__VERSION__</string>
    <key>CFBundleShortVersionString</key>
    <string>__VERSION__</string>
    <key>CFBundleExecutable</key>
    <string>vizo</string>
    <key>CFBundleIconFile</key>
    <string>vizo.icns</string>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>LSSupportsOpeningDocumentsInPlace</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Image Document</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.image</string>
                <string>public.camera-raw-image</string>
                <string>public.jpeg</string>
                <string>public.png</string>
                <string>public.tiff</string>
                <string>org.webmproject.webp</string>
                <string>com.compuserve.gif</string>
                <string>com.microsoft.bmp</string>
                <string>public.heic</string>
                <string>public.heif</string>
                <string>public.avif</string>
            </array>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>jpg</string>
                <string>jpeg</string>
                <string>jpe</string>
                <string>png</string>
                <string>webp</string>
                <string>gif</string>
                <string>bmp</string>
                <string>tiff</string>
                <string>tif</string>
                <string>heic</string>
                <string>heif</string>
                <string>avif</string>
                <string>nef</string>
                <string>cr2</string>
                <string>cr3</string>
                <string>arw</string>
                <string>dng</string>
                <string>rw2</string>
                <string>orf</string>
                <string>pef</string>
                <string>sr2</string>
                <string>raf</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

sed -i.bak "s/__VERSION__/$VERSION/g" "$APP_BUNDLE/Contents/Info.plist"
rm -f "$APP_BUNDLE/Contents/Info.plist.bak"

# Ad-hoc codesign to avoid Gatekeeper execution blocks
if command -v codesign >/dev/null 2>&1; then
    echo "   -> Performing ad-hoc codesign on Vizo.app..."
    codesign --force --deep --sign - "$APP_BUNDLE" >/dev/null 2>&1 || true
fi

# Refresh LaunchServices database cache if tool exists
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
if [ -x "$LSREGISTER" ]; then
    echo "   -> Registering Vizo.app with LaunchServices..."
    "$LSREGISTER" -f "$APP_BUNDLE" >/dev/null 2>&1 || true
fi

echo "   -> Created $APP_BUNDLE"

# 4. Create DMG Installer and Standalone Tarball
echo "💿 [4/4] Generating DMG installer and standalone archives..."
DMG_FILE="$DIST_DIR/vizo-${VERSION}-macos-${ARCH}.dmg"
DMG_LATEST="$DIST_DIR/vizo-macos-${ARCH}.dmg"
rm -f "$DMG_FILE" "$DMG_LATEST"

if command -v hdiutil >/dev/null 2>&1; then
    DMG_STAGING="/tmp/vizo_dmg_staging"
    rm -rf "$DMG_STAGING"
    mkdir -p "$DMG_STAGING"
    
    cp -R "$APP_BUNDLE" "$DMG_STAGING/"
    ln -s /Applications "$DMG_STAGING/Applications"
    cp README.md "$DMG_STAGING/"
    
    echo "   -> Building DMG with hdiutil..."
    hdiutil create -volname "Vizo Image Viewer" \
                   -srcfolder "$DMG_STAGING" \
                   -ov \
                   -format UDZO \
                   "$DMG_FILE" >/dev/null
    
    cp "$DMG_FILE" "$DMG_LATEST"
    rm -rf "$DMG_STAGING"
    echo "   -> Created $DMG_FILE"
    echo "   -> Created $DMG_LATEST"
fi

# Standalone tarball (contains Vizo.app + CLI binary + README)
STANDALONE_TAR="$DIST_DIR/vizo-${VERSION}-macos-${ARCH}.tar.gz"
STANDALONE_LATEST="$DIST_DIR/vizo-macos-${ARCH}.tar.gz"
STANDALONE_DIR="/tmp/vizo_mac_standalone"
rm -rf "$STANDALONE_DIR"
mkdir -p "$STANDALONE_DIR/vizo-macos-${ARCH}"

cp -R "$APP_BUNDLE" "$STANDALONE_DIR/vizo-macos-${ARCH}/"
cp target/release/vizo "$STANDALONE_DIR/vizo-macos-${ARCH}/"
cp README.md install.sh uninstall.sh "$STANDALONE_DIR/vizo-macos-${ARCH}/"

tar -czf "$STANDALONE_TAR" -C "$STANDALONE_DIR" "vizo-macos-${ARCH}"
cp "$STANDALONE_TAR" "$STANDALONE_LATEST"
rm -rf "$STANDALONE_DIR"
echo "   -> Created $STANDALONE_TAR"
echo "   -> Created $STANDALONE_LATEST"

echo ""
echo "✅ macOS packaging completed successfully:"
ls -lh "$DIST_DIR"/*macos* "$DIST_DIR"/Vizo.app
