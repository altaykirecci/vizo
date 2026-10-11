//! Camera RAW format decoding (NEF, CR2, SR2, ARW, DNG, etc.).
//!
//! Uses `rawloader` to unpack RAW sensor pixels, normalizes black/white levels,
//! applies white balance coefficients, and produces a standard RGBA8 image buffer.

use crate::decoder::DecodedImage;
use crate::error::{ViewerError, ViewerResult};
use rawloader::RawImageData;
use std::path::Path;

/// Decodes camera RAW files (such as Nikon NEF, Canon CR2, Sony SR2/ARW, Adobe DNG).
///
/// Handles sensor black/white level normalization, white balance correction,
/// and develops CFA/RGB channels into an 8-bit RGBA buffer.
pub fn decode_raw(path: &Path) -> ViewerResult<DecodedImage> {
    let raw = rawloader::decode_file(path).map_err(|e| ViewerError::RawDecode {
        path: path.to_path_buf(),
        message: format!("{:?}", e),
    })?;

    let width = raw.width;
    let height = raw.height;
    let cpp = raw.cpp;

    if width == 0 || height == 0 {
        return Err(ViewerError::RawDecode {
            path: path.to_path_buf(),
            message: "Decoded RAW image has zero dimensions".to_string(),
        });
    }

    // Default white and black level baselines
    let black = raw.blacklevels[0] as f32;
    let white = if raw.whitelevels[0] as f32 > black {
        raw.whitelevels[0] as f32
    } else {
        65535.0
    };
    let range = (white - black).max(1.0);

    // Camera white balance coefficients (R, G, B)
    let wb_r = if raw.wb_coeffs[0] > 0.0 { raw.wb_coeffs[0] } else { 1.0 };
    let wb_g = if raw.wb_coeffs[1] > 0.0 { raw.wb_coeffs[1] } else { 1.0 };
    let wb_b = if raw.wb_coeffs[2] > 0.0 { raw.wb_coeffs[2] } else { 1.0 };

    let total_pixels = width * height;
    let mut rgba_buffer = Vec::with_capacity(total_pixels * 4);

    match raw.data {
        RawImageData::Integer(data) => {
            if cpp == 3 {
                // 3-channel RGB data
                for chunk in data.chunks_exact(3) {
                    let r = normalize_and_gamma(chunk[0] as f32, black, range, wb_r);
                    let g = normalize_and_gamma(chunk[1] as f32, black, range, wb_g);
                    let b = normalize_and_gamma(chunk[2] as f32, black, range, wb_b);
                    rgba_buffer.push(r);
                    rgba_buffer.push(g);
                    rgba_buffer.push(b);
                    rgba_buffer.push(255);
                }
            } else if cpp == 1 {
                // Single-channel Bayer CFA sensor data.
                // Perform 2x2 grid local interpolation for robust and fast RGB reconstruction.
                for y in 0..height {
                    for x in 0..width {
                        let idx = y * width + x;
                        let val = data.get(idx).copied().unwrap_or(0) as f32;

                        // Standard RGGB pattern classification
                        let is_even_row = y % 2 == 0;
                        let is_even_col = x % 2 == 0;

                        let (r, g, b) = match (is_even_row, is_even_col) {
                            (true, true) => {
                                // Red site: sample neighbor greens and blues
                                let g_val = neighbor_avg_int(&data, width, height, x, y, &[(0, 1), (1, 0)]);
                                let b_val = neighbor_avg_int(&data, width, height, x, y, &[(1, 1)]);
                                (val, g_val, b_val)
                            }
                            (true, false) => {
                                // Green site (row 0, col 1): sample red and blue neighbors
                                let r_val = neighbor_avg_int(&data, width, height, x, y, &[(0, 1)]);
                                let b_val = neighbor_avg_int(&data, width, height, x, y, &[(1, 0)]);
                                (r_val, val, b_val)
                            }
                            (false, true) => {
                                // Green site (row 1, col 0): sample red and blue neighbors
                                let r_val = neighbor_avg_int(&data, width, height, x, y, &[(1, 0)]);
                                let b_val = neighbor_avg_int(&data, width, height, x, y, &[(0, 1)]);
                                (r_val, val, b_val)
                            }
                            (false, false) => {
                                // Blue site: sample red and green neighbors
                                let r_val = neighbor_avg_int(&data, width, height, x, y, &[(1, 1)]);
                                let g_val = neighbor_avg_int(&data, width, height, x, y, &[(0, 1), (1, 0)]);
                                (r_val, g_val, val)
                            }
                        };

                        rgba_buffer.push(normalize_and_gamma(r, black, range, wb_r));
                        rgba_buffer.push(normalize_and_gamma(g, black, range, wb_g));
                        rgba_buffer.push(normalize_and_gamma(b, black, range, wb_b));
                        rgba_buffer.push(255);
                    }
                }
            } else {
                // Fallback for custom channels: duplicate to greyscale
                for &val in &data {
                    let v = normalize_and_gamma(val as f32, black, range, 1.0);
                    rgba_buffer.push(v);
                    rgba_buffer.push(v);
                    rgba_buffer.push(v);
                    rgba_buffer.push(255);
                }
            }
        }
        RawImageData::Float(data) => {
            if cpp == 3 {
                for chunk in data.chunks_exact(3) {
                    let r = normalize_and_gamma(chunk[0], black, range, wb_r);
                    let g = normalize_and_gamma(chunk[1], black, range, wb_g);
                    let b = normalize_and_gamma(chunk[2], black, range, wb_b);
                    rgba_buffer.push(r);
                    rgba_buffer.push(g);
                    rgba_buffer.push(b);
                    rgba_buffer.push(255);
                }
            } else {
                for &val in &data {
                    let v = normalize_and_gamma(val, black, range, 1.0);
                    rgba_buffer.push(v);
                    rgba_buffer.push(v);
                    rgba_buffer.push(v);
                    rgba_buffer.push(255);
                }
            }
        }
    }

    let format_desc = format!(
        "RAW ({}, {}x{})",
        raw.clean_model, width, height
    );

    Ok(DecodedImage {
        width,
        height,
        rgba: rgba_buffer,
        format_name: format_desc,
    })
}

/// Normalizes raw sensor sample, applies white balance multiplier and standard sRGB gamma 2.2.
#[inline]
fn normalize_and_gamma(val: f32, black: f32, range: f32, wb: f32) -> u8 {
    let norm = ((val - black) / range).clamp(0.0, 1.0) * wb;
    let clamped = norm.clamp(0.0, 1.0);
    // Standard sRGB gamma approximation (~2.2)
    let gamma_corrected = clamped.powf(1.0 / 2.2);
    (gamma_corrected * 255.0).round() as u8
}

/// Computes the average value of valid neighboring pixels for Bayer interpolation.
#[inline]
fn neighbor_avg_int(
    data: &[u16],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    offsets: &[(i32, i32)],
) -> f32 {
    let mut sum = 0.0;
    let mut count = 0.0;
    for &(dx, dy) in offsets {
        let nx = (x as i32 + dx).clamp(0, width as i32 - 1) as usize;
        let ny = (y as i32 + dy).clamp(0, height as i32 - 1) as usize;
        let idx = ny * width + nx;
        if let Some(&v) = data.get(idx) {
            sum += v as f32;
            count += 1.0;
        }
    }
    if count > 0.0 {
        sum / count
    } else {
        0.0
    }
}
