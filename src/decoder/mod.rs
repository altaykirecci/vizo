//! Unified image decoding interface.
//!
//! Dispatches image file decoding to either the standard raster engine
//! or the specialized camera RAW decoder depending on file format extension.

pub mod raster;
pub mod raw;

use crate::error::{ViewerError, ViewerResult};
use eframe::egui::ColorImage;
use std::path::Path;

/// In-memory representation of an unmultiplied 8-bit RGBA decoded image.
#[derive(Debug, Clone)]
pub struct DecodedImage {
    /// Image width in pixels.
    pub width: usize,
    /// Image height in pixels.
    pub height: usize,
    /// Raw unmultiplied RGBA8 pixel byte sequence (length = width * height * 4).
    pub rgba: Vec<u8>,
    /// Human-readable format descriptor (e.g. "JPEG", "TIFF", "RAW (Nikon D850, 8256x5504)").
    pub format_name: String,
}

impl DecodedImage {
    /// Converts decoded pixel buffer into an `egui::ColorImage` suitable for GPU texture upload.
    pub fn to_color_image(&self) -> ColorImage {
        ColorImage::from_rgba_unmultiplied([self.width, self.height], &self.rgba)
    }

    /// Returns human-readable aspect ratio string (e.g. "16:9", "3:2", "4:3").
    pub fn aspect_ratio_str(&self) -> String {
        if self.height == 0 {
            return "N/A".to_string();
        }
        let ratio = self.width as f32 / self.height as f32;
        format!("{:.2}:1", ratio)
    }
}

/// Known Camera RAW file extensions.
const RAW_EXTENSIONS: &[&str] = &[
    "nef", "cr2", "cr3", "crw", "sr2", "arw", "dng", "orf", "rw2", "pef", "raf",
];

/// Checks if a file path corresponds to a camera RAW format.
pub fn is_raw_format(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_lowercase();
            RAW_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

/// Decodes any supported image file from disk without swallowing errors.
///
/// Automatically routes Camera RAW formats to the RAW engine and standard
/// bitmap/vector raster formats to the standard raster engine.
pub fn decode_image(path: &Path) -> ViewerResult<DecodedImage> {
    if !path.exists() {
        return Err(ViewerError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Image file does not exist: {}", path.display()),
        )));
    }

    if is_raw_format(path) {
        raw::decode_raw(path)
    } else {
        raster::decode_raster(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_is_raw_format() {
        assert!(is_raw_format(Path::new("test.NEF")));
        assert!(is_raw_format(Path::new("test.cr2")));
        assert!(is_raw_format(Path::new("test.sr2")));
        assert!(is_raw_format(Path::new("test.DNG")));
        assert!(!is_raw_format(Path::new("test.jpg")));
        assert!(!is_raw_format(Path::new("test.png")));
        assert!(!is_raw_format(Path::new("test.tif")));
    }

    #[test]
    fn test_decode_synthetic_png() {
        let temp_dir = std::env::temp_dir().join("oprimage_test_decoder");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let img_path = temp_dir.join("sample.png");
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(10, 20);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]); // Solid red
        }
        img.save(&img_path).unwrap();

        let decoded = decode_image(&img_path).expect("Failed to decode synthetic PNG");
        assert_eq!(decoded.width, 10);
        assert_eq!(decoded.height, 20);
        assert_eq!(decoded.rgba.len(), 10 * 20 * 4);
        assert_eq!(decoded.rgba[0..4], [255, 0, 0, 255]);
        assert_eq!(decoded.aspect_ratio_str(), "0.50:1");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_decode_nonexistent_file_returns_error() {
        let nonexistent = Path::new("/path/that/does/not/exist/image.jpg");
        let result = decode_image(nonexistent);
        assert!(result.is_err(), "Decoding non-existent file must return error");
        match result.unwrap_err() {
            ViewerError::Io(_) => {}
            other => panic!("Expected ViewerError::Io, got: {:?}", other),
        }
    }
}
