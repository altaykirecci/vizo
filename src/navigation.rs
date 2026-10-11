//! Directory navigation and image file discovery.
//!
//! Handles scanning directories for supported image formats, sorting files,
//! and moving forward/backward through the image set.

use crate::error::{ViewerError, ViewerResult};
use std::path::{Path, PathBuf};

/// List of recognized file extensions for standard raster and camera RAW formats.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    // Standard raster formats
    "jpg", "jpeg", "png", "webp", "tif", "tiff", "bmp", "gif", "ico", "tga", "avif", "qoi",
    "hdr", "exr",
    // HEIF / HEIC formats
    "heic", "heif",
    // Camera RAW formats
    "nef", "cr2", "cr3", "crw", "sr2", "arw", "dng", "orf", "rw2", "pef", "raf",
];

/// Checks whether a given file path has a supported image extension.
pub fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_lowercase();
            SUPPORTED_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

/// State tracking the current directory, list of image paths, and current selection index.
#[derive(Debug, Clone, Default)]
pub struct NavigationState {
    /// Active parent directory.
    current_dir: Option<PathBuf>,
    /// Sorted list of image file paths located in the active directory.
    image_paths: Vec<PathBuf>,
    /// Index of the currently displayed image in `image_paths`.
    current_index: usize,
}

impl NavigationState {
    /// Creates an empty navigation state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes navigation from a specific image file path.
    ///
    /// Scans the parent folder of the file, discovers all sibling images,
    /// sorts them alphabetically, and sets the active index to the specified file.
    pub fn from_file(file_path: &Path) -> ViewerResult<Self> {
        let parent = file_path
            .parent()
            .ok_or_else(|| ViewerError::NoImagesInDirectory(file_path.to_path_buf()))?;

        let mut nav = Self::from_directory(parent)?;

        // Find the index of the requested target file
        if let Some(target_pos) = nav.image_paths.iter().position(|p| p == file_path) {
            nav.current_index = target_pos;
        }

        Ok(nav)
    }

    /// Scans a directory and builds an image list.
    pub fn from_directory(dir_path: &Path) -> ViewerResult<Self> {
        if !dir_path.is_dir() {
            return Err(ViewerError::NoImagesInDirectory(dir_path.to_path_buf()));
        }

        let mut entries = Vec::new();
        let read_dir = std::fs::read_dir(dir_path)?;

        for entry in read_dir {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && is_supported_image(&path) {
                entries.push(path);
            }
        }

        if entries.is_empty() {
            return Err(ViewerError::NoImagesInDirectory(dir_path.to_path_buf()));
        }

        // Sort files naturally (case-insensitive filename)
        entries.sort_by(|a, b| {
            let name_a = a.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let name_b = b.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            name_a.cmp(&name_b)
        });

        Ok(Self {
            current_dir: Some(dir_path.to_path_buf()),
            image_paths: entries,
            current_index: 0,
        })
    }

    /// Returns the path to the currently active image, if any.
    pub fn current_file(&self) -> Option<&PathBuf> {
        self.image_paths.get(self.current_index)
    }

    /// Returns the total count of images in the current directory.
    pub fn total_images(&self) -> usize {
        self.image_paths.len()
    }

    /// Returns the 1-based index and total count as a tuple `(current, total)`.
    pub fn position_info(&self) -> (usize, usize) {
        if self.image_paths.is_empty() {
            (0, 0)
        } else {
            (self.current_index + 1, self.image_paths.len())
        }
    }

    /// Advances to the next image in the folder. Loops back to the beginning if at the end.
    pub fn next(&mut self) -> Option<&PathBuf> {
        if self.image_paths.is_empty() {
            return None;
        }

        self.current_index = (self.current_index + 1) % self.image_paths.len();
        self.current_file()
    }

    /// Moves to the previous image in the folder. Loops to the last image if at the beginning.
    pub fn previous(&mut self) -> Option<&PathBuf> {
        if self.image_paths.is_empty() {
            return None;
        }

        if self.current_index == 0 {
            self.current_index = self.image_paths.len() - 1;
        } else {
            self.current_index -= 1;
        }
        self.current_file()
    }

    /// Sets the active index directly.
    pub fn set_index(&mut self, index: usize) -> ViewerResult<&PathBuf> {
        if index >= self.image_paths.len() {
            return Err(ViewerError::IndexOutOfBounds {
                index,
                total: self.image_paths.len(),
            });
        }
        self.current_index = index;
        Ok(&self.image_paths[self.current_index])
    }

    /// Returns the active parent directory path.
    pub fn current_dir(&self) -> Option<&PathBuf> {
        self.current_dir.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_supported_extensions() {
        assert!(is_supported_image(Path::new("photo.NEF")));
        assert!(is_supported_image(Path::new("image.jpg")));
        assert!(is_supported_image(Path::new("graphic.PNG")));
        assert!(is_supported_image(Path::new("scan.TIF")));
        assert!(is_supported_image(Path::new("iphone.heic")));
        assert!(is_supported_image(Path::new("live.HEIF")));
        assert!(is_supported_image(Path::new("camera.cr2")));
        assert!(is_supported_image(Path::new("sony.SR2")));
        assert!(!is_supported_image(Path::new("doc.pdf")));
        assert!(!is_supported_image(Path::new("script.sh")));
    }

    #[test]
    fn test_navigation_cycling() {
        let temp_dir = std::env::temp_dir().join("vizo_test_nav");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        File::create(temp_dir.join("img_a.jpg")).unwrap();
        File::create(temp_dir.join("img_b.png")).unwrap();
        File::create(temp_dir.join("img_c.nef")).unwrap();

        let mut nav = NavigationState::from_directory(&temp_dir).unwrap();
        assert_eq!(nav.total_images(), 3);
        assert_eq!(nav.position_info(), (1, 3));

        // Advance forward
        nav.next();
        assert_eq!(nav.position_info(), (2, 3));
        nav.next();
        assert_eq!(nav.position_info(), (3, 3));
        nav.next(); // Wrap to first
        assert_eq!(nav.position_info(), (1, 3));

        // Move backward
        nav.previous(); // Wrap to last
        assert_eq!(nav.position_info(), (3, 3));
        nav.previous();
        assert_eq!(nav.position_info(), (2, 3));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
