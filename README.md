# Vizo

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)]()

A high-performance, modular image and camera RAW viewer written in Rust.

Built with **`egui` (eframe)** and hardware-accelerated GPU rendering, Vizo delivers an immersive, distraction-free viewing experience with instant 2D pan/zoom, 90-degree lossless rotations, camera EXIF inspection, automated slideshow with auto-fullscreen, and seamless folder navigation.

---

## Key Features

- **Blazing Fast GPU Rendering**: Hardware-accelerated presentation powered by `egui` and `wgpu`.
- **Comprehensive Format Support**: Standard rasters, Apple HEIC/HEIF photos, and professional camera RAW formats.
- **Automated Slideshow**: Customizable auto-advance intervals (3 to 60 seconds) with stepper input controls, auto-closing setup popup, auto-fullscreen, transient exit toast, and one-key exit with `Esc`.
- **Translucent Floating HUD**: Minimalist, floating toolbar providing quick access to folder browsing, navigation, zoom, rotation, EXIF, slideshow, and help overlays.
- **EXIF & Camera Metadata Inspector**: Non-intrusive translucent overlay displaying camera model, lens, shutter speed, aperture, ISO, exposure compensation, timestamps, dimensions, and GPS coordinates with map links.
- **On-Screen Keyboard Shortcuts Overlay**: Transparent help card showing all keyboard shortcuts and gestures in English (`F1` or `?`).
- **Lossless Transformations & Focal Zoom**: Cursor-centered focal zoom, smooth 2D canvas pan, 1:1 pixel scaling, fit-to-window, and 90° lossless quadrant rotations.
- **Cross-Platform**: Native support and dedicated distribution packages for **macOS** (DMG & App bundle with Finder integration), **Linux** (DEB, RPM, Arch pkg, tarball), and **Windows**.

---

## Supported File Formats

Vizo automatically detects and decodes both standard raster images and raw DSLR / mirrorless camera sensor data:

| Category | File Extensions | Decoder Backend |
| :--- | :--- | :--- |
| **Standard Raster** | JPEG (`.jpg`, `.jpeg`), PNG (`.png`), WebP (`.webp`), TIFF (`.tif`, `.tiff`), BMP (`.bmp`), GIF (`.gif`), ICO (`.ico`), TGA (`.tga`), AVIF (`.avif`), QOI (`.qoi`), OpenEXR (`.exr`), HDR (`.hdr`) | `image-rs` |
| **HEIF / HEIC** | High Efficiency Image Format (`.heic`, `.heif`) | `libheif-rs` |
| **Camera RAW** | Nikon NEF (`.nef`), Canon CR2 / CR3 / CRW (`.cr2`, `.cr3`, `.crw`), Sony SR2 / ARW (`.sr2`, `.arw`), Adobe DNG (`.dng`), Olympus ORF (`.orf`), Panasonic RW2 (`.rw2`), Pentax PEF (`.pef`), Fujifilm RAF (`.raf`) | `rawloader` |

---

## Keyboard Shortcuts & Controls

Every feature is accessible via dedicated keyboard shortcuts:

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + T` | **Toggle Toolbar** | Shows or hides the floating HUD toolbar |
| `Ctrl + I` | **Toggle EXIF** | Shows or hides the camera EXIF & metadata overlay card |
| `Ctrl + S` | **Slideshow** | Opens slideshow configuration popup / toggles playback |
| `F1` / `?` | **Keyboard Help** | Displays the translucent shortcuts help overlay |
| `Ctrl + H` | **About Vizo** | Opens the application information and copyright modal |
| `Esc` | **Exit / Dismiss** | Exits active slideshow (and fullscreen) or closes open overlays |
| `Ctrl + O` / `O` | **Open Image** | Opens native file dialog to select an image file |
| `Ctrl + Shift + O` | **Open Folder** | Opens native folder picker to select an entire image directory |
| `→` / `Ctrl + →` | **Next Image** | Advances to the next supported image in the directory |
| `←` / `Ctrl + ←` | **Previous Image** | Returns to the previous image in the directory |
| `Ctrl + R` | **Rotate Clockwise** | Rotates image 90° clockwise (lossless quadrant rotation) |
| `Ctrl + L` | **Rotate Counter-Clockwise** | Rotates image 90° counter-clockwise |
| `Ctrl + +` / `Ctrl + =` | **Zoom In** | Magnifies image zoom level |
| `Ctrl + -` | **Zoom Out** | Reduces image zoom level |
| `Ctrl + 0` | **100% Actual Size** | Resets scale to 1:1 native pixel resolution |
| `Ctrl + F` | **Fit to Window** | Fits image dimensions into the current window viewport |
| `Double Click` | **Toggle Fit / 100%** | Toggles between Fit to Window and 1:1 Actual Size |
| `Left Click + Drag` | **Pan Image** | Moves the image canvas freely |
| `Mouse Wheel` | **Focal Zoom** | Zooms smoothly centered on mouse cursor position |

---

## Architectural Design

The codebase follows a modular, decoupled architecture with strongly-typed error handling:

```
src/
├── main.rs          # Application entry point & native viewport configuration
├── app.rs           # eframe::App implementation, event coordinator, and render loops
├── about.rs         # Modal dialog showing version, repository & copyright info
├── error.rs         # Strongly-typed error enum (ViewerError) and ViewerResult<T>
├── decoder/         # Unified decoding subsystem
│   ├── mod.rs       # ImageDecoder facade & format classification
│   ├── raster.rs    # Standard raster decoder using image-rs
│   ├── heic.rs      # HEIC/HEIF decoder using libheif-rs
│   └── raw.rs       # Camera RAW CFA unpacker & demosaicing using rawloader
├── help.rs          # Translucent keyboard shortcuts overlay (F1 / ?)
├── macos_ext.rs     # macOS LaunchServices / Finder "Open With" event handler
├── navigation.rs    # Directory scanner, natural sorting, next/previous traversal
├── shortcuts.rs     # Centralized keyboard shortcut dispatcher
├── slideshow.rs     # Slideshow timer, stepper interval popup, and exit toast
├── toolbar.rs       # Translucent floating HUD toolbar (Ctrl + T)
├── transform.rs     # 2D pan, zoom factor, 90° quadrant rotations, fit-to-screen
└── viewer.rs        # Viewport canvas, GPU mesh rendering, and error presentation
```

---

## Installation

### macOS

#### Pre-built Application Bundle or DMG
- Mount `vizo-*-macos-arm64.dmg` or extract `vizo-macos-arm64.tar.gz`.
- Drag **Vizo.app** into `/Applications`.
- Vizo is registered with macOS LaunchServices to seamlessly open images via Finder's **"Open With"** and double-click.

#### From Source on macOS
```bash
./install.sh
```
This builds an optimized release binary, packages `/Applications/Vizo.app`, and installs the CLI binary to `~/.local/bin/vizo`.

---

### Linux

#### Quick Install (Current User)
Installs `vizo` binary to `~/.local/bin`, installs application icon to `~/.local/share/icons`, and registers the desktop launcher in `~/.local/share/applications`:

```bash
./install.sh
```

#### System-Wide Install
```bash
sudo ./install.sh --system
```

#### Distribution Packages
Pre-built packages are generated under `dist/`:
- **Debian / Ubuntu**: `sudo dpkg -i dist/vizo_*_amd64.deb`
- **Fedora / RHEL**: `sudo rpm -Uvh dist/vizo-*.rpm`
- **Arch Linux**: Run `makepkg -si` in the project root.
- **Universal Tarball**: Extract `dist/vizo-linux-x86_64.tar.gz` and run `vizo`.

---

## Building & Packaging from Source

### Prerequisites
- **Rust toolchain** (1.75 or later): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **libheif** development libraries:
  - macOS: `brew install libheif`
  - Ubuntu/Debian: `sudo apt-get install libheif-dev`
  - Fedora: `sudo dnf install libheif-devel`
  - Arch: `sudo pacman -S libheif`

### Build & Run
```bash
# Fast check and tests
./scripts/quick_check.sh

# Build optimized binary
cargo build --release

# Run
cargo run --release -- /path/to/image.jpg
```

### Full Automated Verification & Distribution Build
To run the automated test suite, compile the release binary, update local installations, and build all native packages across platforms:

```bash
./scripts/verify_and_package.sh
```

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

Copyright (c) 2026 **Altay Kireççi**.
