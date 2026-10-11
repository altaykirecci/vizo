//! HEIC / HEIF format decoding using `libheif-rs`.
//!
//! Extracts primary image handle, decodes RGBA planes, and returns
//! standard unmultiplied 8-bit RGBA buffers.

use crate::decoder::DecodedImage;
use crate::error::{ViewerError, ViewerResult};
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::path::Path;

/// Decodes HEIC and HEIF image files.
pub fn decode_heic(path: &Path) -> ViewerResult<DecodedImage> {
    let lib_heif = LibHeif::new();
    let path_str = path.to_str().ok_or_else(|| ViewerError::HeicDecode {
        path: path.to_path_buf(),
        message: "Path contains invalid UTF-8 characters".to_string(),
    })?;

    let ctx = HeifContext::read_from_file(path_str).map_err(|e| ViewerError::HeicDecode {
        path: path.to_path_buf(),
        message: format!("{:?}", e),
    })?;

    let handle = ctx
        .primary_image_handle()
        .map_err(|e| ViewerError::HeicDecode {
            path: path.to_path_buf(),
            message: format!("{:?}", e),
        })?;

    let width = handle.width() as usize;
    let height = handle.height() as usize;

    let image = lib_heif
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None)
        .map_err(|e| ViewerError::HeicDecode {
            path: path.to_path_buf(),
            message: format!("{:?}", e),
        })?;

    let planes = image.planes();
    let interleaved = planes
        .interleaved
        .ok_or_else(|| ViewerError::HeicDecode {
            path: path.to_path_buf(),
            message: "Missing interleaved RGBA plane in decoded HEIC".to_string(),
        })?;

    let stride = interleaved.stride;
    let data = interleaved.data;

    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        let row_start = y * stride;
        let row_end = row_start + (width * 4);
        if row_end <= data.len() {
            rgba.extend_from_slice(&data[row_start..row_end]);
        }
    }

    Ok(DecodedImage {
        width,
        height,
        rgba,
        format_name: format!("HEIC ({}x{})", width, height),
    })
}
