//! Unified image decoding interface.
//!
//! Dispatches image file decoding to either the standard raster engine
//! or the specialized camera RAW decoder depending on file format extension.

pub mod heic;
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

/// Known HEIC/HEIF file extensions.
const HEIC_EXTENSIONS: &[&str] = &["heic", "heif"];

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

/// Checks if a file path corresponds to an HEIC / HEIF format.
pub fn is_heic_format(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_lowercase();
            HEIC_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

/// Applies EXIF orientation (1..=8) to an RGBA8 byte buffer.
///
/// Automatically transforms pixels (rotations and flips) so the image is rendered upright.
pub fn apply_orientation_to_rgba(
    width: usize,
    height: usize,
    rgba: Vec<u8>,
    orientation: Option<u32>,
) -> (usize, usize, Vec<u8>) {
    let Some(orient_val) = orientation else {
        return (width, height, rgba);
    };
    let Some(orient) = image::metadata::Orientation::from_exif(orient_val as u8) else {
        return (width, height, rgba);
    };
    if orient == image::metadata::Orientation::NoTransforms {
        return (width, height, rgba);
    }

    if width * height * 4 != rgba.len() {
        return (width, height, rgba);
    }

    let Some(img_buf) = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
        width as u32,
        height as u32,
        rgba,
    ) else {
        return (width, height, Vec::new());
    };

    let mut dynamic_img = image::DynamicImage::ImageRgba8(img_buf);
    dynamic_img.apply_orientation(orient);

    let new_w = dynamic_img.width() as usize;
    let new_h = dynamic_img.height() as usize;
    let new_rgba = dynamic_img.into_rgba8().into_raw();

    (new_w, new_h, new_rgba)
}

/// Decodes any supported image file from disk without swallowing errors.
///
/// Automatically routes Camera RAW formats to the RAW engine, HEIC/HEIF to the
/// libheif engine, and standard bitmap/vector raster formats to the raster engine.
pub fn decode_image(path: &Path) -> ViewerResult<DecodedImage> {
    if !path.exists() {
        return Err(ViewerError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Image file does not exist: {}", path.display()),
        )));
    }

    if is_raw_format(path) {
        raw::decode_raw(path)
    } else if is_heic_format(path) {
        heic::decode_heic(path)
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
    fn test_is_heic_format() {
        assert!(is_heic_format(Path::new("photo.HEIC")));
        assert!(is_heic_format(Path::new("photo.heif")));
        assert!(!is_heic_format(Path::new("photo.jpg")));
        assert!(!is_heic_format(Path::new("photo.png")));
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
    fn test_apply_orientation_to_rgba() {
        // 10 width, 20 height solid RGBA image
        let rgba = vec![255u8; 10 * 20 * 4];

        // Orientation None -> unchanged
        let (w, h, _) = apply_orientation_to_rgba(10, 20, rgba.clone(), None);
        assert_eq!((w, h), (10, 20));

        // Orientation 1 (Normal) -> unchanged
        let (w, h, _) = apply_orientation_to_rgba(10, 20, rgba.clone(), Some(1));
        assert_eq!((w, h), (10, 20));

        // Orientation 6 (Rotate 90 CW) -> width & height swapped
        let (w, h, _) = apply_orientation_to_rgba(10, 20, rgba.clone(), Some(6));
        assert_eq!((w, h), (20, 10));

        // Orientation 8 (Rotate 270 CW) -> width & height swapped
        let (w, h, _) = apply_orientation_to_rgba(10, 20, rgba.clone(), Some(8));
        assert_eq!((w, h), (20, 10));

        // Orientation 3 (Rotate 180) -> dimensions same
        let (w, h, _) = apply_orientation_to_rgba(10, 20, rgba.clone(), Some(3));
        assert_eq!((w, h), (10, 20));
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
