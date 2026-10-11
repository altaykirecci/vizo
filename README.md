# Vizo

A high-performance, modular image and camera RAW viewer written in Rust.

Built with **`egui` (eframe)** and hardware-accelerated rendering, Vizo delivers an immersive, distraction-free viewing experience with instant 2D pan/zoom, 90-degree lossless rotations, camera EXIF inspection, and seamless folder navigation.

---

## Supported File Formats

Vizo automatically detects and decodes both standard raster images and raw DSLR / mirrorless camera sensor data:

| Category | File Extensions |
| :--- | :--- |
| **Standard Raster** | JPEG (`.jpg`, `.jpeg`), PNG (`.png`), WebP (`.webp`), TIFF (`.tif`, `.tiff`), BMP (`.bmp`), GIF (`.gif`), ICO (`.ico`), TGA (`.tga`), AVIF (`.avif`), QOI (`.qoi`), OpenEXR (`.exr`), HDR (`.hdr`) |
| **Camera RAW** | Nikon NEF (`.nef`), Canon CR2 / CR3 (`.cr2`, `.cr3`), Sony SR2 / ARW (`.sr2`, `.arw`), Adobe DNG (`.dng`), Olympus ORF (`.orf`), Panasonic RW2 (`.rw2`), Pentax PEF (`.pef`), Fujifilm RAF (`.raf`) |

---

## Architectural Design

The codebase follows a clean, decoupled, and modular architecture with strictly typed error handling:

```
src/
├── main.rs          # Application entry point & native viewport configuration
├── app.rs           # eframe::App implementation & top-level coordinator
├── about.rs         # Modal dialog showing version, repository & copyright info
├── error.rs         # Strongly-typed error enum (ViewerError) using thiserror
├── decoder/         # Unified decoding subsystem
│   ├── mod.rs       # ImageDecoder facade & format classification
│   ├── raster.rs    # Standard raster decoder using image-rs
│   └── raw.rs       # Camera RAW CFA unpacker & demosaicing using rawloader
├── navigation.rs    # Directory scanner, natural sorting, next/previous traversal
├── transform.rs     # 2D pan, zoom factor, 90° quadrant rotations, fit-to-screen
├── toolbar.rs       # Translucent floating HUD toolbar (Ctrl + T)
├── exif_info.rs     # Camera hardware and exposure metadata extractor (Ctrl + I)
├── shortcuts.rs     # Centralized keyboard shortcut dispatcher
└── viewer.rs        # Viewport canvas, mesh rendering, and error banner presentation
```

---

## Keyboard Shortcuts

Every feature is accessible via dedicated keyboard shortcuts:

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + T` | **Toggle Toolbar** | Shows or hides the floating HUD toolbar |
| `Ctrl + I` | **Toggle EXIF** | Shows or hides the EXIF exposure and hardware details card |
| `Ctrl + H` / `F1` | **About** | Opens the application information and copyright dialog |
| `Ctrl + O` / `O` | **Open Image** | Opens native file picker to select an image file |
| `Ctrl + Shift + O` | **Open Folder** | Opens native directory picker to select an entire image folder |
| `Ctrl + Right` / `→` | **Next Image** | Advances to the next supported image in the folder |
| `Ctrl + Left` / `←` | **Previous Image** | Returns to the previous image in the folder |
| `Ctrl + R` | **Rotate Clockwise** | Rotates 90° clockwise (lossless UV transformation) |
| `Ctrl + L` | **Rotate Counter-Clockwise** | Rotates 90° counter-clockwise |
| `Ctrl + +` / `Ctrl + =` | **Zoom In** | Magnifies image by 25% |
| `Ctrl + -` | **Zoom Out** | Reduces image magnification by 25% |
| `Ctrl + 0` | **100% Actual Size** | Resets scale to 1:1 native pixel resolution |
| `Ctrl + F` | **Fit to Window** | Fits image dimensions into the available viewport |
| `Double Click` | **Quick Scale Toggle** | Toggles between Fit to Window and 1:1 Actual Size |
| `Mouse Drag` | **Pan** | Moves the image canvas freely |
| `Mouse Wheel` | **Focal Zoom** | Zooms smoothly centered on mouse cursor position |

---

## Linux Installation

### Quick Install (Current User)
Installs `vizo` binary to `~/.local/bin`, adds desktop launcher to `~/.local/share/applications`, and installs application icons:

```bash
./install.sh
```

### System-Wide Install
```bash
sudo ./install.sh --system
```

### Arch Linux (PKGBUILD)
```bash
makepkg -si
```

---

## License & Copyright

**Altay Kireççi (c)(p)10112026**

Licensed under the MIT License or Apache License 2.0.
