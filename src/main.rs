//! Vizo Image Viewer
//!
//! High-performance, modular image and Camera RAW viewer written in Rust.
//! Supports standard raster (JPEG, PNG, WebP, TIFF, GIF, BMP) and RAW camera formats
//! (NEF, CR2, SR2, ARW, DNG, etc.) with 2D pan/zoom, 90° rotations, and EXIF overlay.

pub mod about;
pub mod app;
pub mod decoder;
pub mod error;
pub mod exif_info;
pub mod help;
pub mod macos_ext;
pub mod navigation;
pub mod shortcuts;
pub mod slideshow;
pub mod toolbar;
pub mod transform;
pub mod viewer;

use app::VizoViewer;
use eframe::egui::Vec2;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    // Initialize macOS LaunchServices / Finder file event handler
    macos_ext::init_macos_file_handler();

    // Collect command line arguments (e.g. `vizo sample.nef`)
    // Filter out macOS Finder process serial number argument (-psn_...) if launched via GUI
    let initial_file = std::env::args()
        .nth(1)
        .filter(|arg| !arg.starts_with("-psn"))
        .map(PathBuf::from);

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Vizo Image Viewer")
            .with_inner_size(Vec2::new(1280.0, 800.0))
            .with_min_inner_size(Vec2::new(640.0, 480.0))
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Vizo Image Viewer",
        native_options,
        Box::new(move |cc| Ok(Box::new(VizoViewer::new(cc, initial_file)))),
    )
}
