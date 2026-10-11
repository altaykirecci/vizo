//! Camera RAW format decoding (NEF, CR2, SR2, ARW, DNG, etc.).
//!
//! Provides a two-tier decoding pipeline:
//! 1. High-resolution embedded camera-developed JPEG extraction (instant decoding with
//!    authentic camera color science, tone curves, and white balance).
//! 2. Calibrated Bayer CFA sensor demosaicing using `rawloader`, protected against panics
//!    with `std::panic::catch_unwind`, dynamic CFA pattern detection (`color_at`),
//!    green-normalized white balance, and sRGB gamma curve mapping.
//! 3. Resilient fallback to any embedded preview if sensor demosaicing fails or panics.

use crate::decoder::DecodedImage;
use crate::error::{ViewerError, ViewerResult};
use rawloader::RawImageData;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Decodes camera RAW files (Nikon NEF, Canon CR2, Sony SR2/ARW, Adobe DNG, etc.).
pub fn decode_raw(path: &Path) -> ViewerResult<DecodedImage> {
    // Read raw file bytes for embedded preview check or rawloader decoding
    let mut file = File::open(path).map_err(ViewerError::Io)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(ViewerError::Io)?;

    // 1. Attempt to extract full/high-resolution embedded JPEG (>= 1.0 MP) developed by the camera
    let raw_decoded = if let Some(embedded) = try_extract_embedded_jpeg(&bytes, 1_000_000) {
        embedded
    } else {
        // 2. Fall back to rawloader Bayer sensor demosaicing (with panic protection)
        match decode_with_rawloader(path) {
            Ok(decoded) => decoded,
            Err(err) => {
                // 3. Resilient fallback: attempt to extract any embedded preview (>= 40,000 pixels)
                if let Some(fallback_embedded) = try_extract_embedded_jpeg(&bytes, 40_000) {
                    fallback_embedded
                } else {
                    return Err(err);
                }
            }
        }
    };

    // Auto-orient based on camera EXIF orientation tag if present
    let orientation = crate::exif_info::read_exif_orientation(path);
    let (oriented_w, oriented_h, oriented_rgba) = crate::decoder::apply_orientation_to_rgba(
        raw_decoded.width,
        raw_decoded.height,
        raw_decoded.rgba,
        orientation,
    );

    Ok(DecodedImage {
        width: oriented_w,
        height: oriented_h,
        rgba: oriented_rgba,
        format_name: raw_decoded.format_name,
    })
}

/// Scans raw byte stream for camera-developed preview JPEGs satisfying `min_area`.
fn try_extract_embedded_jpeg(bytes: &[u8], min_area: usize) -> Option<DecodedImage> {
    let len = bytes.len();
    let mut pos = 0;
    let mut best_img: Option<(usize, usize, Vec<u8>, usize)> = None;

    while pos + 3 < len {
        // Look for JPEG Start of Image (SOI) marker: 0xFF, 0xD8, 0xFF
        if bytes[pos] == 0xFF && bytes[pos + 1] == 0xD8 && bytes[pos + 2] == 0xFF {
            let start = pos;
            let mut search = pos + 2;
            let mut found_end = None;

            // Look for JPEG End of Image (EOI) marker: 0xFF, 0xD9
            while search + 1 < len {
                if bytes[search] == 0xFF && bytes[search + 1] == 0xD9 {
                    found_end = Some(search + 2);
                    break;
                }
                search += 1;
            }

            if let Some(end) = found_end {
                let seg_len = end - start;
                // Previews of meaningful resolution are typically >= 40 KB
                if seg_len > 40_000 {
                    if let Ok(dyn_img) = image::load_from_memory(&bytes[start..end]) {
                        let w = dyn_img.width() as usize;
                        let h = dyn_img.height() as usize;
                        let area = w * h;

                        if area >= min_area {
                            if best_img.as_ref().map(|(_, _, _, a)| area > *a).unwrap_or(true) {
                                best_img = Some((w, h, dyn_img.to_rgba8().into_raw(), area));
                            }
                        }
                    }
                }
                pos = end;
                continue;
            }
        }
        pos += 1;
    }

    best_img.map(|(width, height, rgba, _)| DecodedImage {
        width,
        height,
        rgba,
        format_name: format!("RAW (Camera Preview, {}x{})", width, height),
    })
}

/// Decodes sensor Bayer/RGB data directly using `rawloader`, protected against internal panics.
fn decode_with_rawloader(path: &Path) -> ViewerResult<DecodedImage> {
    let path_buf = path.to_path_buf();

    // rawloader may panic on corrupt or unexpected TIFF tags; catch unwinds safely
    let decode_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        rawloader::decode_file(&path_buf)
    }));

    let raw = match decode_result {
        Ok(Ok(raw)) => raw,
        Ok(Err(e)) => {
            return Err(ViewerError::RawDecode {
                path: path.to_path_buf(),
                message: format!("{:?}", e),
            });
        }
        Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Internal panic inside rawloader decoder".to_string()
            };
            return Err(ViewerError::RawDecode {
                path: path.to_path_buf(),
                message: format!("Rawloader panic intercepted: {}", msg),
            });
        }
    };

    let width = raw.width;
    let height = raw.height;
    let cpp = raw.cpp;

    if width == 0 || height == 0 {
        return Err(ViewerError::RawDecode {
            path: path.to_path_buf(),
            message: "Decoded RAW image has zero dimensions".to_string(),
        });
    }

    let black = raw.blacklevels[0] as f32;
    let white = if raw.whitelevels[0] as f32 > black {
        raw.whitelevels[0] as f32
    } else {
        65535.0
    };
    let range = (white - black).max(1.0);

    // Normalize white balance coefficients relative to green channel (index 1).
    // Some raw formats report unnormalized multipliers (e.g. 477.0, 256.0, 467.0)
    // which blow out images to solid white if not normalized by green reference.
    let g_ref = if raw.wb_coeffs[1] > 0.0 { raw.wb_coeffs[1] } else { 1.0 };
    let wb_r = if raw.wb_coeffs[0] > 0.0 {
        (raw.wb_coeffs[0] / g_ref).max(0.1)
    } else {
        1.0
    };
    let wb_g = 1.0;
    let wb_b = if raw.wb_coeffs[2] > 0.0 {
        (raw.wb_coeffs[2] / g_ref).max(0.1)
    } else {
        1.0
    };

    let total_pixels = width * height;
    let mut rgba_buffer = vec![0u8; total_pixels * 4];

    match raw.data {
        RawImageData::Integer(ref data) => {
            if cpp == 1 {
                // Bayer Color Filter Array (CFA) sensor data.
                // CFAColor: 0 = Red, 1 = Green, 2 = Blue.
                for y in 0..height {
                    for x in 0..width {
                        let idx = y * width + x;
                        let val = data[idx] as f32;
                        let c = raw.cfa.color_at(y, x);

                        let (r, g, b) = match c {
                            0 => {
                                // Red site: interpolate green from cross neighbors, blue from diagonals
                                let mut g_sum = 0.0;
                                let mut g_cnt = 0.0;
                                for &(dy, dx) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                                    let ny = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                                    let nx = (x as isize + dx).clamp(0, width as isize - 1) as usize;
                                    g_sum += data[ny * width + nx] as f32;
                                    g_cnt += 1.0;
                                }
                                let mut b_sum = 0.0;
                                let mut b_cnt = 0.0;
                                for &(dy, dx) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                                    let ny = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                                    let nx = (x as isize + dx).clamp(0, width as isize - 1) as usize;
                                    b_sum += data[ny * width + nx] as f32;
                                    b_cnt += 1.0;
                                }
                                (val, g_sum / g_cnt, b_sum / b_cnt)
                            }
                            1 => {
                                // Green site: check whether horizontal neighbors are Red or Blue
                                let left_x = (x as isize - 1).clamp(0, width as isize - 1) as usize;
                                let right_x = (x as isize + 1).clamp(0, width as isize - 1) as usize;
                                let left_c = raw.cfa.color_at(y, left_x);
                                let right_c = raw.cfa.color_at(y, right_x);

                                let (r_val, b_val) = if left_c == 0 || right_c == 0 {
                                    // Red is horizontal, Blue is vertical
                                    let r_sum = (data[y * width + left_x] + data[y * width + right_x]) as f32 / 2.0;
                                    let top_y = (y as isize - 1).clamp(0, height as isize - 1) as usize;
                                    let bot_y = (y as isize + 1).clamp(0, height as isize - 1) as usize;
                                    let b_sum = (data[top_y * width + x] + data[bot_y * width + x]) as f32 / 2.0;
                                    (r_sum, b_sum)
                                } else {
                                    // Blue is horizontal, Red is vertical
                                    let b_sum = (data[y * width + left_x] + data[y * width + right_x]) as f32 / 2.0;
                                    let top_y = (y as isize - 1).clamp(0, height as isize - 1) as usize;
                                    let bot_y = (y as isize + 1).clamp(0, height as isize - 1) as usize;
                                    let r_sum = (data[top_y * width + x] + data[bot_y * width + x]) as f32 / 2.0;
                                    (r_sum, b_sum)
                                };
                                (r_val, val, b_val)
                            }
                            2 => {
                                // Blue site: interpolate green from cross neighbors, red from diagonals
                                let mut g_sum = 0.0;
                                let mut g_cnt = 0.0;
                                for &(dy, dx) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                                    let ny = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                                    let nx = (x as isize + dx).clamp(0, width as isize - 1) as usize;
                                    g_sum += data[ny * width + nx] as f32;
                                    g_cnt += 1.0;
                                }
                                let mut r_sum = 0.0;
                                let mut r_cnt = 0.0;
                                for &(dy, dx) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                                    let ny = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                                    let nx = (x as isize + dx).clamp(0, width as isize - 1) as usize;
                                    r_sum += data[ny * width + nx] as f32;
                                    r_cnt += 1.0;
                                }
                                (r_sum / r_cnt, g_sum / g_cnt, val)
                            }
                            _ => (val, val, val),
                        };

                        let norm_r = ((r - black) / range).clamp(0.0, 1.0) * wb_r;
                        let norm_g = ((g - black) / range).clamp(0.0, 1.0) * wb_g;
                        let norm_b = ((b - black) / range).clamp(0.0, 1.0) * wb_b;

                        let out_idx = idx * 4;
                        rgba_buffer[out_idx] = (norm_r.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                        rgba_buffer[out_idx + 1] = (norm_g.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                        rgba_buffer[out_idx + 2] = (norm_b.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                        rgba_buffer[out_idx + 3] = 255;
                    }
                }
            } else if cpp == 3 {
                for (chunk, out_chunk) in data.chunks_exact(3).zip(rgba_buffer.chunks_exact_mut(4)) {
                    let r = ((chunk[0] as f32 - black) / range).clamp(0.0, 1.0) * wb_r;
                    let g = ((chunk[1] as f32 - black) / range).clamp(0.0, 1.0) * wb_g;
                    let b = ((chunk[2] as f32 - black) / range).clamp(0.0, 1.0) * wb_b;
                    out_chunk[0] = (r.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[1] = (g.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[2] = (b.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[3] = 255;
                }
            } else {
                for (&val, out_chunk) in data.iter().zip(rgba_buffer.chunks_exact_mut(4)) {
                    let v = ((val as f32 - black) / range).clamp(0.0, 1.0);
                    let byte = (v.powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[0] = byte;
                    out_chunk[1] = byte;
                    out_chunk[2] = byte;
                    out_chunk[3] = 255;
                }
            }
        }
        RawImageData::Float(ref data) => {
            if cpp == 3 {
                for (chunk, out_chunk) in data.chunks_exact(3).zip(rgba_buffer.chunks_exact_mut(4)) {
                    let r = ((chunk[0] - black) / range).clamp(0.0, 1.0) * wb_r;
                    let g = ((chunk[1] - black) / range).clamp(0.0, 1.0) * wb_g;
                    let b = ((chunk[2] - black) / range).clamp(0.0, 1.0) * wb_b;
                    out_chunk[0] = (r.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[1] = (g.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[2] = (b.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[3] = 255;
                }
            } else {
                for (&val, out_chunk) in data.iter().zip(rgba_buffer.chunks_exact_mut(4)) {
                    let v = ((val - black) / range).clamp(0.0, 1.0);
                    let byte = (v.powf(1.0 / 2.2) * 255.0).round() as u8;
                    out_chunk[0] = byte;
                    out_chunk[1] = byte;
                    out_chunk[2] = byte;
                    out_chunk[3] = 255;
                }
            }
        }
    }

    let format_desc = format!("RAW ({}, {}x{})", raw.clean_model, width, height);

    Ok(DecodedImage {
        width,
        height,
        rgba: rgba_buffer,
        format_name: format_desc,
    })
}
