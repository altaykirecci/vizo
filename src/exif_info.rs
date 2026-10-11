//! EXIF and file metadata extraction and overlay card rendering.
//!
//! Parses camera exposure settings (shutter speed, aperture, ISO, focal length),
//! camera/lens hardware info, and renders an overlay HUD.

use crate::error::{ViewerError, ViewerResult};
use eframe::egui::{self, Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke};
use exif::{In, Reader as ExifReader, Tag, Value};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Aggregated metadata extracted from an image file and its EXIF header.
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    /// File name with extension.
    pub filename: String,
    /// Absolute parent directory path.
    pub folder: String,
    /// Human-readable file size string (e.g. "24.5 MB").
    pub file_size: String,
    /// Image width in pixels.
    pub width: Option<u32>,
    /// Image height in pixels.
    pub height: Option<u32>,
    /// Camera manufacturer (e.g. "Nikon", "Sony", "Canon").
    pub make: Option<String>,
    /// Camera model (e.g. "Nikon Z7 II", "ILCE-7RM4").
    pub model: Option<String>,
    /// Lens description.
    pub lens: Option<String>,
    /// Shutter speed (e.g. "1/250s", "2.5s").
    pub exposure_time: Option<String>,
    /// Aperture F-number (e.g. "f/2.8").
    pub f_number: Option<String>,
    /// ISO sensitivity (e.g. "ISO 100").
    pub iso: Option<String>,
    /// Focal length (e.g. "50mm").
    pub focal_length: Option<String>,
    /// Capture timestamp.
    pub date_time: Option<String>,
}

/// Reads EXIF tags and file attributes from the specified file path.
pub fn extract_exif(path: &Path) -> ViewerResult<ExifData> {
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let folder = path
        .parent()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let metadata = std::fs::metadata(path).map_err(ViewerError::Io)?;
    let size_bytes = metadata.len();
    let file_size = format_file_size(size_bytes);

    let mut info = ExifData {
        filename,
        folder,
        file_size,
        ..Default::default()
    };

    // Attempt to open file and parse EXIF stream
    let file = File::open(path).map_err(ViewerError::Io)?;
    let mut buf_reader = BufReader::new(file);

    if let Ok(exif) = ExifReader::new().read_from_container(&mut buf_reader) {
        if let Some(field) = exif.get_field(Tag::Make, In::PRIMARY) {
            info.make = Some(field.display_value().to_string().trim().to_string());
        }
        if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
            info.model = Some(field.display_value().to_string().trim().to_string());
        }
        if let Some(field) = exif.get_field(Tag::LensModel, In::PRIMARY) {
            info.lens = Some(field.display_value().to_string().trim().to_string());
        }
        if let Some(field) = exif.get_field(Tag::ExposureTime, In::PRIMARY) {
            info.exposure_time = Some(format!("{}s", field.display_value()));
        }
        if let Some(field) = exif.get_field(Tag::FNumber, In::PRIMARY) {
            info.f_number = Some(format!("f/{}", field.display_value()));
        }
        if let Some(field) = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY) {
            info.iso = Some(format!("ISO {}", field.display_value()));
        }
        if let Some(field) = exif.get_field(Tag::FocalLength, In::PRIMARY) {
            info.focal_length = Some(format!("{}mm", field.display_value()));
        }
        if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
            info.date_time = Some(field.display_value().to_string());
        }
        if let Some(field) = exif.get_field(Tag::PixelXDimension, In::PRIMARY) {
            if let Value::Long(ref v) = field.value {
                if let Some(&w) = v.first() {
                    info.width = Some(w);
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::PixelYDimension, In::PRIMARY) {
            if let Value::Long(ref v) = field.value {
                if let Some(&h) = v.first() {
                    info.height = Some(h);
                }
            }
        }
    }

    Ok(info)
}

/// Formats byte count into human-readable B, KB, MB, or GB.
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Renders a semi-transparent HUD card with EXIF info at the top-right corner of the canvas.
pub fn render_exif_overlay(
    ctx: &egui::Context,
    data: &ExifData,
    viewport_rect: Rect,
) {
    let card_width = 320.0_f32;
    let margin = 20.0_f32;
    let card_pos = Pos2::new(
        viewport_rect.max.x - card_width - margin,
        viewport_rect.min.y + 60.0_f32,
    );

    let area_id = egui::Id::new("exif_overlay_card");

    egui::Area::new(area_id)
        .fixed_pos(card_pos)
        .show(ctx, |ui| {
            Frame::none()
                .fill(Color32::from_black_alpha(200))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(40)))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(Margin::same(14.0_f32))
                .show(ui, |ui| {
                    ui.set_width(card_width);
                    ui.vertical(|ui| {
                        // Title header
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("ℹ EXIF & File Details")
                                    .color(Color32::from_rgb(220, 220, 230))
                                    .strong(),
                            );
                        });
                        ui.separator();

                        // File info
                        render_info_row(ui, "File", &data.filename);
                        render_info_row(ui, "Size", &data.file_size);

                        if let (Some(w), Some(h)) = (data.width, data.height) {
                            render_info_row(ui, "Dimensions", &format!("{} × {} px", w, h));
                        }

                        // Hardware info
                        if let Some(ref model) = data.model {
                            let cam = data
                                .make
                                .as_ref()
                                .map(|m| format!("{} {}", m, model))
                                .unwrap_or_else(|| model.clone());
                            render_info_row(ui, "Camera", &cam);
                        }

                        if let Some(ref lens) = data.lens {
                            render_info_row(ui, "Lens", lens);
                        }

                        // Exposure settings
                        let mut exposure_parts = Vec::new();
                        if let Some(ref exp) = data.exposure_time {
                            exposure_parts.push(exp.clone());
                        }
                        if let Some(ref f) = data.f_number {
                            exposure_parts.push(f.clone());
                        }
                        if let Some(ref iso) = data.iso {
                            exposure_parts.push(iso.clone());
                        }
                        if !exposure_parts.is_empty() {
                            render_info_row(ui, "Exposure", &exposure_parts.join("  |  "));
                        }

                        if let Some(ref focal) = data.focal_length {
                            render_info_row(ui, "Focal Length", focal);
                        }

                        if let Some(ref dt) = data.date_time {
                            render_info_row(ui, "Date Taken", dt);
                        }
                    });
                });
        });
}

fn render_info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{}:", label))
                .color(Color32::from_rgb(150, 150, 160))
                .size(12.0_f32),
        );
        ui.label(
            egui::RichText::new(value)
                .color(Color32::from_rgb(230, 230, 240))
                .size(12.0_f32),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024 * 5), "5.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024 * 2), "2.00 GB");
    }

    #[test]
    fn test_extract_exif_fallback() {
        let temp_dir = std::env::temp_dir().join("oprimage_test_exif");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let dummy_file = temp_dir.join("dummy.png");
        std::fs::write(&dummy_file, b"header_data").unwrap();

        let info = extract_exif(&dummy_file).expect("Failed to extract metadata");
        assert_eq!(info.filename, "dummy.png");
        assert_eq!(info.file_size, "11 B");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
