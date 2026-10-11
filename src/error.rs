//! Error types for the Vizo image viewer application.
//!
//! Provides strongly-typed error enumeration covering file I/O, image decoding,
//! camera RAW processing, EXIF metadata extraction, and directory navigation.

use std::path::PathBuf;
use thiserror::Error;

/// Master error type for the image viewer application.
#[derive(Debug, Error)]
pub enum ViewerError {
    /// Standard file system I/O error.
    #[error("File system I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Image crate decoding error for standard raster formats (JPEG, PNG, TIFF, etc.).
    #[error("Image decoding error: {0}")]
    ImageDecode(#[from] image::ImageError),

    /// Camera RAW decoding error (NEF, CR2, SR2, DNG, etc.).
    #[error("RAW camera decoding failed for '{path}': {message}")]
    RawDecode {
        path: PathBuf,
        message: String,
    },

    /// HEIC / HEIF decoding error.
    #[error("HEIC decoding failed for '{path}': {message}")]
    HeicDecode {
        path: PathBuf,
        message: String,
    },

    /// EXIF metadata extraction or parsing error.
    #[error("EXIF metadata error for '{path}': {message}")]
    ExifDecode {
        path: PathBuf,
        message: String,
    },

    /// Specified directory does not contain any supported image files.
    #[error("No supported images found in directory: {0}")]
    NoImagesInDirectory(PathBuf),

    /// Navigation index out of bounds.
    #[error("Image index out of bounds: index {index}, total {total}")]
    IndexOutOfBounds {
        index: usize,
        total: usize,
    },

    /// Unsupported image format extension.
    #[error("Unsupported image file format: {0}")]
    UnsupportedFormat(String),

    /// File dialog was cancelled by user.
    #[error("File selection was cancelled")]
    SelectionCancelled,
}

/// Convenience Result alias using `ViewerError`.
pub type ViewerResult<T> = Result<T, ViewerError>;
