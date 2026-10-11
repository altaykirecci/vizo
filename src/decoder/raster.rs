//! Standard raster image decoding (JPEG, PNG, WebP, TIFF, BMP, GIF, etc.).
//!
//! Utilizes the `image` crate with strict error propagation and no silent fallbacks.

use crate::decoder::DecodedImage;
use crate::error::{ViewerError, ViewerResult};
use image::ImageReader;
use std::path::Path;

/// Decodes standard raster images (JPEG, PNG, WebP, TIFF, BMP, GIF, etc.).
///
/// Converts decoded pixels to an unmultiplied RGBA8 buffer.
/// Returns a descriptive `ViewerError` if reading, format detection, or decoding fails.
pub fn decode_raster(path: &Path) -> ViewerResult<DecodedImage> {
    // Open the file with buffered I/O
    let reader = ImageReader::open(path)
        .map_err(|e| ViewerError::Io(e))?
        .with_guessed_format()
        .map_err(|e| ViewerError::Io(e))?;

    let format_desc = reader
        .format()
        .map(|f| format!("{:?}", f))
        .unwrap_or_else(|| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("Unknown")
                .to_uppercase()
        });

    // Decode image into DynamicImage
    let mut dynamic_img = reader.decode().map_err(ViewerError::ImageDecode)?;

    // Automatically orient image based on EXIF orientation tag if present
    if let Some(orient_val) = crate::exif_info::read_exif_orientation(path) {
        if let Some(orient) = image::metadata::Orientation::from_exif(orient_val as u8) {
            dynamic_img.apply_orientation(orient);
        }
    }

    let width = dynamic_img.width() as usize;
    let height = dynamic_img.height() as usize;
    let rgba8 = dynamic_img.to_rgba8();
    let raw_bytes = rgba8.into_raw();

    Ok(DecodedImage {
        width,
        height,
        rgba: raw_bytes,
        format_name: format_desc,
    })
}
