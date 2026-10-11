//! View transformations for pan, zoom, and 90-degree stepped rotations.
//!
//! Provides coordinate manipulation to smoothly display images on screen,
//! inspect details at 1:1 pixel scale, fit to viewport dimensions, and rotate.

use eframe::egui::{Pos2, Rect, Vec2};

/// Minimum allowed zoom factor (5%).
pub const MIN_ZOOM: f32 = 0.05;
/// Maximum allowed zoom factor (5000%).
pub const MAX_ZOOM: f32 = 50.0;
/// Zoom step factor multiplier for keyboard/toolbar actions.
pub const ZOOM_STEP_FACTOR: f32 = 1.25;

/// State for 2D pan, zoom, and 90-degree quadrant rotation.
#[derive(Debug, Clone)]
pub struct ViewTransform {
    /// Current magnification factor (1.0 = 100% original scale).
    pub zoom: f32,
    /// 2D pan offset in screen coordinates.
    pub pan: Vec2,
    /// Number of 90-degree clockwise quadrants (0 = 0°, 1 = 90°, 2 = 180°, 3 = 270°).
    pub rotation_quadrant: u8,
}

impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            rotation_quadrant: 0,
        }
    }
}

impl ViewTransform {
    /// Creates a default transform with 100% zoom, centered pan, and 0° rotation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Rotates 90 degrees clockwise.
    pub fn rotate_clockwise(&mut self) {
        self.rotation_quadrant = (self.rotation_quadrant + 1) % 4;
    }

    /// Rotates 90 degrees counter-clockwise.
    pub fn rotate_counter_clockwise(&mut self) {
        self.rotation_quadrant = (self.rotation_quadrant + 3) % 4;
    }

    /// Returns the effective angle in radians.
    pub fn rotation_radians(&self) -> f32 {
        (self.rotation_quadrant as f32) * std::f32::consts::FRAC_PI_2
    }

    /// Returns the effective angle in degrees (0, 90, 180, 270).
    pub fn rotation_degrees(&self) -> u16 {
        (self.rotation_quadrant as u16) * 90
    }

    /// Zooms in by the standard step factor.
    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom * ZOOM_STEP_FACTOR);
    }

    /// Zooms out by the standard step factor.
    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom / ZOOM_STEP_FACTOR);
    }

    /// Directly sets zoom level, clamped between `MIN_ZOOM` and `MAX_ZOOM`.
    pub fn set_zoom(&mut self, new_zoom: f32) {
        self.zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Zooms in or out centered at a specific focal point (e.g. mouse cursor position).
    pub fn zoom_towards(&mut self, factor: f32, focus_point: Pos2, canvas_rect: Rect) {
        let old_zoom = self.zoom;
        let new_zoom = (old_zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        if (new_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }

        let canvas_center = canvas_rect.center();
        // Vector from canvas center to current focus point
        let rel_focus = focus_point - (canvas_center + self.pan);

        // Adjust pan offset so that the pixel under the focus point remains stable
        let scale_ratio = new_zoom / old_zoom;
        self.pan += rel_focus * (1.0 - scale_ratio);
        self.zoom = new_zoom;
    }

    /// Sets scale to 1.0 (100% native pixel size) and centers pan offset.
    pub fn actual_size(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
    }

    /// Calculates zoom and centers the image so it fits completely within the viewport.
    pub fn fit_to_viewport(&mut self, image_size: Vec2, viewport_size: Vec2) {
        if image_size.x <= 0.0 || image_size.y <= 0.0 || viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
            return;
        }

        // Account for 90 or 270 degree rotation swapping width and height
        let (effective_w, effective_h) = if self.rotation_quadrant % 2 == 1 {
            (image_size.y, image_size.x)
        } else {
            (image_size.x, image_size.y)
        };

        let scale_x = viewport_size.x / effective_w;
        let scale_y = viewport_size.y / effective_h;
        let best_fit = scale_x.min(scale_y);

        self.zoom = best_fit.clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = Vec2::ZERO;
    }

    /// Resets pan and zoom, but preserves current rotation.
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_cycling() {
        let mut t = ViewTransform::new();
        assert_eq!(t.rotation_degrees(), 0);

        t.rotate_clockwise();
        assert_eq!(t.rotation_degrees(), 90);

        t.rotate_clockwise();
        assert_eq!(t.rotation_degrees(), 180);

        t.rotate_clockwise();
        assert_eq!(t.rotation_degrees(), 270);

        t.rotate_clockwise();
        assert_eq!(t.rotation_degrees(), 0);

        t.rotate_counter_clockwise();
        assert_eq!(t.rotation_degrees(), 270);
    }

    #[test]
    fn test_zoom_clamping() {
        let mut t = ViewTransform::new();
        t.set_zoom(100.0);
        assert_eq!(t.zoom, MAX_ZOOM);

        t.set_zoom(0.001);
        assert_eq!(t.zoom, MIN_ZOOM);
    }

    #[test]
    fn test_fit_to_viewport() {
        let mut t = ViewTransform::new();
        let img = Vec2::new(1000.0, 500.0);
        let viewport = Vec2::new(500.0, 500.0);

        t.fit_to_viewport(img, viewport);
        assert_eq!(t.zoom, 0.5);
        assert_eq!(t.pan, Vec2::ZERO);

        // When rotated 90 degrees, dimensions invert
        t.rotate_clockwise();
        t.fit_to_viewport(img, viewport);
        // rotated img is 500w x 1000h, fitting into 500x500 means zoom = 500/1000 = 0.5
        assert_eq!(t.zoom, 0.5);
    }
}
