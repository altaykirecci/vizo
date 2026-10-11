//! EXIF and file metadata extraction and overlay card rendering.
//!
//! Parses camera exposure settings (shutter speed, aperture, ISO, focal length),
//! GPS coordinates (latitude, longitude, altitude), capture timestamps (date taken),
//! camera/lens hardware info, and renders an overlay HUD.

use crate::error::{ViewerError, ViewerResult};
use eframe::egui::{self, Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke};
use exif::{In, Reader as ExifReader, Tag, Value};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Geolocation GPS coordinates and map details.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GpsInfo {
    /// Latitude in signed decimal degrees (e.g. 41.013667).
    pub latitude: f64,
    /// Longitude in signed decimal degrees (e.g. 28.977000).
    pub longitude: f64,
    /// Latitude in degrees, minutes, seconds format (e.g. "41° 00' 49.2\" N").
    pub latitude_dms: String,
    /// Longitude in degrees, minutes, seconds format (e.g. "28° 58' 37.2\" E").
    pub longitude_dms: String,
    /// Altitude in meters if available (e.g. "125.4 m").
    pub altitude: Option<String>,
    /// Google Maps direct link with coordinates.
    pub map_url: String,
}

/// Aggregated metadata extracted from an image file and its EXIF header.
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    // --- File & Storage ---
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
    /// Calculated megapixels (e.g. "24.2 MP").
    pub megapixels: Option<String>,

    // --- Hardware & Firmware ---
    /// Camera manufacturer (e.g. "Nikon", "Sony", "Canon").
    pub make: Option<String>,
    /// Camera model (e.g. "Nikon Z7 II", "ILCE-7RM4").
    pub model: Option<String>,
    /// Lens description.
    pub lens: Option<String>,
    /// Software or camera firmware version.
    pub software: Option<String>,

    // --- Shooting & Exposure Parameters ---
    /// Shutter speed (e.g. "1/250s", "2.5s").
    pub exposure_time: Option<String>,
    /// Aperture F-number (e.g. "f/2.8").
    pub f_number: Option<String>,
    /// ISO sensitivity (e.g. "ISO 100").
    pub iso: Option<String>,
    /// Focal length (e.g. "50mm").
    pub focal_length: Option<String>,
    /// 35mm equivalent focal length (e.g. "75mm (35mm eq.)").
    pub focal_length_35mm: Option<String>,
    /// Exposure program (e.g. "Aperture priority", "Manual").
    pub exposure_program: Option<String>,
    /// Exposure bias / compensation (e.g. "+0.3 EV", "-0.7 EV").
    pub exposure_bias: Option<String>,
    /// Metering mode (e.g. "Pattern", "Spot", "Center weighted average").
    pub metering_mode: Option<String>,
    /// White balance mode (e.g. "Auto white balance", "Manual").
    pub white_balance: Option<String>,
    /// Flash fired status (e.g. "Flash did not fire").
    pub flash: Option<String>,

    // --- Timestamps ---
    /// Primary capture timestamp (Date Taken).
    pub date_taken: Option<String>,
    /// Fallback capture timestamp for backwards compatibility.
    pub date_time: Option<String>,

    // --- Geolocation ---
    /// Geolocation GPS coordinates if embedded in image.
    pub gps: Option<GpsInfo>,

    // --- Orientation ---
    /// Raw EXIF orientation tag value (1..=8).
    pub orientation: Option<u32>,

    // --- Author & Description ---
    /// Photographer or creator name.
    pub artist: Option<String>,
    /// Copyright statement.
    pub copyright: Option<String>,
    /// Image title or description.
    pub description: Option<String>,
}

/// Helper function to find a tag in PRIMARY IFD or across any parsed IFDs.
fn find_tag<'a>(exif: &'a exif::Exif, tag: Tag) -> Option<&'a exif::Field> {
    exif.get_field(tag, In::PRIMARY)
        .or_else(|| exif.fields().find(|f| f.tag == tag))
}

/// Parses EXIF Rational degree/minute/second array and direction reference into decimal degrees and DMS string.
pub fn parse_coordinate(coord_val: &Value, ref_str: Option<&str>) -> Option<(f64, String)> {
    if let Value::Rational(ref rationals) = coord_val {
        if rationals.len() >= 3 {
            let deg = rationals[0].to_f64();
            let min = rationals[1].to_f64();
            let sec = rationals[2].to_f64();
            let mut decimal = deg + (min / 60.0) + (sec / 3600.0);

            let ref_clean = ref_str.unwrap_or("").trim().to_uppercase();
            if ref_clean == "S" || ref_clean == "W" {
                decimal = -decimal;
            }

            let dms = if !ref_clean.is_empty() {
                format!("{:.0}° {:02.0}' {:04.1}\" {}", deg, min, sec, ref_clean)
            } else {
                format!("{:.0}° {:02.0}' {:04.1}\"", deg, min, sec)
            };

            return Some((decimal, dms));
        }
    }
    None
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
        // --- Hardware ---
        if let Some(field) = find_tag(&exif, Tag::Make) {
            info.make = Some(field.display_value().to_string().trim().to_string());
        }
        if let Some(field) = find_tag(&exif, Tag::Model) {
            info.model = Some(field.display_value().to_string().trim().to_string());
        }
        if let Some(field) = find_tag(&exif, Tag::LensModel) {
            info.lens = Some(field.display_value().to_string().trim().to_string());
        }
        if let Some(field) = find_tag(&exif, Tag::Software) {
            let sw = field.display_value().to_string().trim().to_string();
            if !sw.is_empty() {
                info.software = Some(sw);
            }
        }

        // --- Exposure & Camera Settings ---
        if let Some(field) = find_tag(&exif, Tag::ExposureTime) {
            info.exposure_time = Some(format!("{}s", field.display_value()));
        }
        if let Some(field) = find_tag(&exif, Tag::FNumber) {
            info.f_number = Some(format!("f/{}", field.display_value()));
        }
        if let Some(field) = find_tag(&exif, Tag::PhotographicSensitivity) {
            info.iso = Some(format!("ISO {}", field.display_value()));
        }
        if let Some(field) = find_tag(&exif, Tag::FocalLength) {
            info.focal_length = Some(format!("{}mm", field.display_value()));
        }
        if let Some(field) = find_tag(&exif, Tag::FocalLengthIn35mmFilm) {
            let val = field.display_value().to_string().trim().to_string();
            if !val.is_empty() && val != "0" {
                info.focal_length_35mm = Some(format!("{}mm", val));
            }
        }
        if let Some(field) = find_tag(&exif, Tag::ExposureProgram) {
            let desc = field.display_value().to_string().trim().to_string();
            if !desc.is_empty() && desc != "Not defined" {
                info.exposure_program = Some(desc);
            }
        }
        if let Some(field) = find_tag(&exif, Tag::ExposureBiasValue) {
            let val_str = field.display_value().to_string().trim().to_string();
            if let Ok(ev) = val_str.parse::<f64>() {
                if ev.abs() < 0.001 {
                    info.exposure_bias = Some("0 EV".to_string());
                } else if ev > 0.0 {
                    info.exposure_bias = Some(format!("+{:.2} EV", ev));
                } else {
                    info.exposure_bias = Some(format!("{:.2} EV", ev));
                }
            } else if !val_str.is_empty() {
                info.exposure_bias = Some(format!("{} EV", val_str));
            }
        }
        if let Some(field) = find_tag(&exif, Tag::MeteringMode) {
            let mode = field.display_value().to_string().trim().to_string();
            if !mode.is_empty() && mode != "Unknown" {
                info.metering_mode = Some(mode);
            }
        }
        if let Some(field) = find_tag(&exif, Tag::WhiteBalance) {
            let wb = field.display_value().to_string().trim().to_string();
            if !wb.is_empty() {
                info.white_balance = Some(wb);
            }
        }
        if let Some(field) = find_tag(&exif, Tag::Flash) {
            let flash_desc = field.display_value().to_string().trim().to_string();
            if !flash_desc.is_empty() {
                info.flash = Some(flash_desc);
            }
        }

        // --- Date Taken (Timestamps) ---
        let date_original = find_tag(&exif, Tag::DateTimeOriginal)
            .map(|f| f.display_value().to_string().trim().to_string());
        let offset_time = find_tag(&exif, Tag::OffsetTimeOriginal)
            .or_else(|| find_tag(&exif, Tag::OffsetTime))
            .map(|f| f.display_value().to_string().trim().to_string());

        let date_taken = match (date_original, offset_time) {
            (Some(dt), Some(tz)) if !tz.is_empty() => Some(format!("{} ({})", dt, tz)),
            (Some(dt), _) => Some(dt),
            (None, _) => find_tag(&exif, Tag::DateTimeDigitized)
                .or_else(|| find_tag(&exif, Tag::DateTime))
                .map(|f| f.display_value().to_string().trim().to_string()),
        };
        info.date_taken = date_taken.clone();
        info.date_time = date_taken;

        // --- Dimensions ---
        // In RAW and multi-IFD formats, IFD0 may contain a low-res preview or thumbnail.
        // We scan all tags across all IFDs and take the maximum width and length.
        let mut max_width = 0u32;
        let mut max_height = 0u32;
        for field in exif.fields() {
            match field.tag {
                Tag::PixelXDimension | Tag::ImageWidth => {
                    if let Some(w) = field.value.get_uint(0) {
                        if w > max_width {
                            max_width = w;
                        }
                    }
                }
                Tag::PixelYDimension | Tag::ImageLength => {
                    if let Some(h) = field.value.get_uint(0) {
                        if h > max_height {
                            max_height = h;
                        }
                    }
                }
                _ => {}
            }
        }
        if max_width > 0 {
            info.width = Some(max_width);
        }
        if max_height > 0 {
            info.height = Some(max_height);
        }
        if let (Some(w), Some(h)) = (info.width, info.height) {
            let mp = (w as f64 * h as f64) / 1_000_000.0;
            info.megapixels = Some(format!("{:.1} MP", mp));
        }

        // --- Geolocation (GPS) ---
        let lat_opt = find_tag(&exif, Tag::GPSLatitude).and_then(|f| {
            let lat_ref = find_tag(&exif, Tag::GPSLatitudeRef).map(|r| r.display_value().to_string());
            parse_coordinate(&f.value, lat_ref.as_deref())
        });

        let lon_opt = find_tag(&exif, Tag::GPSLongitude).and_then(|f| {
            let lon_ref = find_tag(&exif, Tag::GPSLongitudeRef).map(|r| r.display_value().to_string());
            parse_coordinate(&f.value, lon_ref.as_deref())
        });

        if let (Some((lat, lat_dms)), Some((lon, lon_dms))) = (lat_opt, lon_opt) {
            let altitude = find_tag(&exif, Tag::GPSAltitude).and_then(|f| {
                if let Value::Rational(ref rat) = f.value {
                    rat.first().map(|r| {
                        let alt_m = r.to_f64();
                        let is_below = find_tag(&exif, Tag::GPSAltitudeRef)
                            .and_then(|r| r.value.get_uint(0))
                            .map(|v| v == 1)
                            .unwrap_or(false);
                        if is_below {
                            format!("-{:.1} m (below sea level)", alt_m)
                        } else {
                            format!("{:.1} m", alt_m)
                        }
                    })
                } else {
                    None
                }
            });

            let map_url = format!("https://www.google.com/maps?q={:.6},{:.6}", lat, lon);

            info.gps = Some(GpsInfo {
                latitude: lat,
                longitude: lon,
                latitude_dms: lat_dms,
                longitude_dms: lon_dms,
                altitude,
                map_url,
            });
        }

        // --- Orientation ---
        if let Some(field) = find_tag(&exif, Tag::Orientation) {
            info.orientation = field.value.get_uint(0);
        }

        // --- Author, Copyright & Description ---
        if let Some(field) = find_tag(&exif, Tag::Artist) {
            let artist = field.display_value().to_string().trim().to_string();
            if !artist.is_empty() {
                info.artist = Some(artist);
            }
        }
        if let Some(field) = find_tag(&exif, Tag::Copyright) {
            let copyright = field.display_value().to_string().trim().to_string();
            if !copyright.is_empty() {
                info.copyright = Some(copyright);
            }
        }
        if let Some(field) = find_tag(&exif, Tag::ImageDescription) {
            let desc = field.display_value().to_string().trim().to_string();
            if !desc.is_empty() {
                info.description = Some(desc);
            }
        }
    }

    Ok(info)
}

/// Reads the raw EXIF orientation tag value (1..=8) from a file path without parsing all metadata.
pub fn read_exif_orientation(path: &Path) -> Option<u32> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = ExifReader::new().read_from_container(&mut reader).ok()?;
    exif.get_field(Tag::Orientation, In::PRIMARY)
        .or_else(|| exif.fields().find(|f| f.tag == Tag::Orientation))
        .and_then(|f| f.value.get_uint(0))
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

/// Renders a highly transparent HUD card with EXIF info at the top-right corner of the canvas.
/// Designed to fit all metadata cleanly without scrolling and with increased transparency.
pub fn render_exif_overlay(
    ctx: &egui::Context,
    data: &ExifData,
    viewport_rect: Rect,
) {
    let card_width = 330.0_f32;
    let margin = 16.0_f32;
    let card_pos = Pos2::new(
        viewport_rect.max.x - card_width - margin,
        viewport_rect.min.y + 55.0_f32,
    );

    let area_id = egui::Id::new("exif_overlay_card");

    egui::Area::new(area_id)
        .fixed_pos(card_pos)
        .show(ctx, |ui| {
            Frame::none()
                // Increased transparency: 130/255 (~50% translucent glass effect)
                .fill(Color32::from_black_alpha(130))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(35)))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(Margin::symmetric(12.0_f32, 10.0_f32))
                .show(ui, |ui| {
                    ui.set_width(card_width);

                    // No ScrollArea - compact fit without vertical scrolling
                    ui.vertical(|ui| {
                        // Title header
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("📷 EXIF & Image Details")
                                    .color(Color32::from_rgb(235, 235, 250))
                                    .size(12.5_f32)
                                    .strong(),
                            );
                        });
                        ui.add_space(2.0);
                        ui.separator();
                        ui.add_space(2.0);

                        // File & size row
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&data.filename)
                                    .color(Color32::from_rgb(240, 240, 255))
                                    .size(11.5_f32)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new(format!("• {}", data.file_size))
                                    .color(Color32::from_rgb(160, 170, 190))
                                    .size(11.0_f32),
                            );
                        });

                        // Dimensions & Megapixels row
                        if let (Some(w), Some(h)) = (data.width, data.height) {
                            let dim_str = if let Some(ref mp) = data.megapixels {
                                format!("{} × {} px ({})", w, h, mp)
                            } else {
                                format!("{} × {} px", w, h)
                            };
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📐").size(11.0_f32));
                                ui.label(
                                    egui::RichText::new(dim_str)
                                        .color(Color32::from_rgb(220, 225, 240))
                                        .size(11.5_f32),
                                );
                            });
                        }

                        // Date Taken row (if present)
                        if let Some(ref dt) = data.date_taken {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📅").size(11.0_f32));
                                ui.label(
                                    egui::RichText::new(dt)
                                        .color(Color32::from_rgb(220, 225, 240))
                                        .size(11.5_f32),
                                );
                            });
                        }

                        // Camera & Lens row (if present)
                        if data.model.is_some() || data.lens.is_some() {
                            let mut cam_parts = Vec::new();
                            if let Some(ref model) = data.model {
                                let cam = data
                                    .make
                                    .as_ref()
                                    .map(|m| format!("{} {}", m, model))
                                    .unwrap_or_else(|| model.clone());
                                cam_parts.push(cam);
                            }
                            if let Some(ref lens) = data.lens {
                                cam_parts.push(lens.clone());
                            }
                            ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new("📷").size(11.0_f32));
                                ui.label(
                                    egui::RichText::new(cam_parts.join(" • "))
                                        .color(Color32::from_rgb(210, 220, 240))
                                        .size(11.5_f32),
                                );
                            });
                        }

                        // Exposure parameters badge row (shutter, aperture, ISO, focal length)
                        let mut exp_pills = Vec::new();
                        if let Some(ref exp) = data.exposure_time {
                            exp_pills.push(exp.clone());
                        }
                        if let Some(ref f) = data.f_number {
                            exp_pills.push(f.clone());
                        }
                        if let Some(ref iso) = data.iso {
                            exp_pills.push(iso.clone());
                        }
                        if let Some(ref focal) = data.focal_length {
                            let fl_str = if let Some(ref eq35) = data.focal_length_35mm {
                                format!("{} ({})", focal, eq35)
                            } else {
                                focal.clone()
                            };
                            exp_pills.push(fl_str);
                        }
                        if !exp_pills.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new("⚙️").size(11.0_f32));
                                for pill in &exp_pills {
                                    render_pill_badge(ui, pill);
                                }
                            });
                        }

                        // Secondary settings badges (EV compensation, WB, Flash, Program)
                        let mut sec_items = Vec::new();
                        if let Some(ref prog) = data.exposure_program {
                            sec_items.push(prog.clone());
                        }
                        if let Some(ref ev) = data.exposure_bias {
                            sec_items.push(format!("EV: {}", ev));
                        }
                        if let Some(ref wb) = data.white_balance {
                            sec_items.push(wb.clone());
                        }
                        if let Some(ref fl) = data.flash {
                            sec_items.push(fl.clone());
                        }
                        if !sec_items.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                for item in &sec_items {
                                    render_subtle_badge(ui, item);
                                }
                            });
                        }

                        // GPS Geolocation row (if present)
                        if let Some(ref gps) = data.gps {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📍").size(11.0_f32));
                                ui.label(
                                    egui::RichText::new(format!("{:.4}°, {:.4}°", gps.latitude, gps.longitude))
                                        .color(Color32::from_rgb(215, 225, 245))
                                        .size(11.5_f32),
                                );
                                ui.hyperlink_to(
                                    egui::RichText::new("🗺 Map")
                                        .color(Color32::from_rgb(110, 190, 255))
                                        .size(11.0_f32),
                                    &gps.map_url,
                                );
                                if ui.small_button("📋").on_hover_text("Copy GPS coordinates").clicked() {
                                    ui.output_mut(|o| {
                                        o.copied_text = format!("{:.6}, {:.6}", gps.latitude, gps.longitude);
                                    });
                                }
                            });
                        }

                        // Author / Copyright row (if present)
                        if let Some(ref cr) = data.copyright.as_ref().or(data.artist.as_ref()) {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("© {}", cr))
                                        .color(Color32::from_rgb(160, 170, 185))
                                        .size(11.0_f32),
                                );
                            });
                        }
                    });
                });
        });
}

fn render_pill_badge(ui: &mut egui::Ui, text: &str) {
    Frame::none()
        .fill(Color32::from_black_alpha(70))
        .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(35)))
        .rounding(Rounding::same(4.0_f32))
        .inner_margin(Margin::symmetric(5.0_f32, 2.0_f32))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .color(Color32::from_rgb(230, 235, 245))
                    .size(11.0_f32)
                    .strong(),
            );
        });
}

fn render_subtle_badge(ui: &mut egui::Ui, text: &str) {
    Frame::none()
        .fill(Color32::from_black_alpha(50))
        .rounding(Rounding::same(3.0_f32))
        .inner_margin(Margin::symmetric(4.0_f32, 1.5_f32))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .color(Color32::from_rgb(175, 185, 205))
                    .size(10.5_f32),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use exif::Rational;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024 * 5), "5.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024 * 2), "2.00 GB");
    }

    #[test]
    fn test_extract_exif_fallback() {
        let temp_dir = std::env::temp_dir().join("vizo_test_exif");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let dummy_file = temp_dir.join("dummy.png");
        std::fs::write(&dummy_file, b"header_data").unwrap();

        let info = extract_exif(&dummy_file).expect("Failed to extract metadata");
        assert_eq!(info.filename, "dummy.png");
        assert_eq!(info.file_size, "11 B");
        assert!(info.gps.is_none());
        assert!(info.orientation.is_none());
        assert_eq!(read_exif_orientation(&dummy_file), None);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_read_exif_orientation_tiff() {
        let temp_dir = std::env::temp_dir().join("vizo_test_exif_orient");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let tiff_file = temp_dir.join("test_orient.tif");
        // Minimal little-endian TIFF with 1 IFD containing Orientation = 6 (Rotate 90 CW)
        #[rustfmt::skip]
        let tiff_bytes: [u8; 26] = [
            b'I', b'I', 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00, // Header (offset to IFD = 8)
            0x01, 0x00,                                     // 1 directory entry
            0x12, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, // Tag 0x0112, Type 3, Count 1, Value 6
            0x00, 0x00, 0x00, 0x00,                         // Next IFD = 0
        ];
        std::fs::write(&tiff_file, &tiff_bytes).unwrap();

        assert_eq!(read_exif_orientation(&tiff_file), Some(6));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_parse_coordinate_north_east() {
        // Istanbul coordinates: 41 deg 0 min 49.2 sec N
        let rat = Value::Rational(vec![
            Rational { num: 41, denom: 1 },
            Rational { num: 0, denom: 1 },
            Rational { num: 492, denom: 10 },
        ]);
        let parsed = parse_coordinate(&rat, Some("N"));
        assert!(parsed.is_some());
        let (dec, dms) = parsed.unwrap();
        assert!((dec - 41.013667).abs() < 0.0001);
        assert!(dms.contains("41°"));
        assert!(dms.contains("N"));
    }

    #[test]
    fn test_parse_coordinate_south_west() {
        // Rio de Janeiro latitude: 22 deg 54 min 36 sec S -> should be negative decimal
        let rat = Value::Rational(vec![
            Rational { num: 22, denom: 1 },
            Rational { num: 54, denom: 1 },
            Rational { num: 36, denom: 1 },
        ]);
        let parsed = parse_coordinate(&rat, Some("S"));
        assert!(parsed.is_some());
        let (dec, dms) = parsed.unwrap();
        assert!(dec < 0.0);
        assert!((dec - (-22.91)).abs() < 0.001);
        assert!(dms.contains("S"));
    }

    #[test]
    fn test_gps_info_map_url() {
        let gps = GpsInfo {
            latitude: 41.013667,
            longitude: 28.977000,
            latitude_dms: "41° 00' 49.2\" N".to_string(),
            longitude_dms: "28° 58' 37.2\" E".to_string(),
            altitude: Some("35.0 m".to_string()),
            map_url: "https://www.google.com/maps?q=41.013667,28.977000".to_string(),
        };
        assert!(gps.map_url.contains("41.013667"));
        assert!(gps.map_url.contains("28.977000"));
        assert_eq!(gps.altitude, Some("35.0 m".to_string()));
    }
}
