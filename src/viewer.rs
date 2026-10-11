//! Canvas viewport rendering and GPU texture presentation.
//!
//! Renders the active image with hardware-accelerated 2D pan, zoom,
//! and 90-degree quadrant rotation using custom mesh UV mapping.

use crate::error::ViewerError;
use crate::shortcuts::AppAction;
use crate::transform::ViewTransform;
use eframe::egui::{
    self, Color32, Mesh, Pos2, Rect, Response, Rounding, Sense, Stroke, TextureHandle, Vec2,
};
use eframe::epaint::Vertex;

/// Renders the central canvas viewport, handling mouse dragging (pan), scrolling (zoom),
/// and the interactive empty-state open panel.
pub fn render_canvas(
    ui: &mut egui::Ui,
    texture: Option<&TextureHandle>,
    transform: &mut ViewTransform,
    last_error: Option<&ViewerError>,
) -> (Response, Option<AppAction>) {
    let available_rect = ui.available_rect_before_wrap();
    let (response, painter) = ui.allocate_painter(available_rect.size(), Sense::click_and_drag());
    let mut triggered_action = None;

    // Fill background with neutral dark theme color that preserves photo color perception
    painter.rect_filled(available_rect, 0.0, Color32::from_rgb(18, 18, 20));

    // Handle mouse drag for panning
    if response.dragged() {
        transform.pan += response.drag_delta();
    }

    // Handle mouse wheel for zooming towards cursor
    let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
    if scroll_delta.abs() > 0.0 {
        let zoom_factor = if scroll_delta > 0.0 { 1.15 } else { 1.0 / 1.15 };
        if let Some(mouse_pos) = response.hover_pos() {
            transform.zoom_towards(zoom_factor, mouse_pos, available_rect);
        }
    }

    // If there is an active error, render prominent error banner in the center
    if let Some(err) = last_error {
        let center = available_rect.center();
        let error_box = Rect::from_center_size(center, Vec2::new(480.0, 160.0));

        painter.rect_filled(error_box, 8.0, Color32::from_rgb(45, 15, 15));
        painter.rect_stroke(
            error_box,
            8.0,
            Stroke::new(1.5_f32, Color32::from_rgb(220, 60, 60)),
        );

        let title_pos = Pos2::new(error_box.min.x + 20.0, error_box.min.y + 25.0);
        painter.text(
            title_pos,
            egui::Align2::LEFT_TOP,
            "⚠ Image Load / Decoding Error",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(255, 120, 120),
        );

        let msg_pos = Pos2::new(error_box.min.x + 20.0, error_box.min.y + 60.0);
        painter.text(
            msg_pos,
            egui::Align2::LEFT_TOP,
            format!("{}", err),
            egui::FontId::proportional(13.0),
            Color32::from_rgb(240, 200, 200),
        );
        return (response, None);
    }

    // If an image texture is loaded, render it with transform
    if let Some(tex) = texture {
        let original_size = tex.size_vec2();

        // Swap dimensions if rotated 90 or 270 degrees
        let is_transposed = transform.rotation_quadrant % 2 == 1;
        let display_size = if is_transposed {
            Vec2::new(original_size.y, original_size.x) * transform.zoom
        } else {
            original_size * transform.zoom
        };

        let canvas_center = available_rect.center() + transform.pan;
        let dst_rect = Rect::from_center_size(canvas_center, display_size);

        // Compute rotated UV coordinates for 90-degree increments
        let (uv_tl, uv_tr, uv_br, uv_bl) = match transform.rotation_quadrant {
            0 => (
                Pos2::new(0.0, 0.0),
                Pos2::new(1.0, 0.0),
                Pos2::new(1.0, 1.0),
                Pos2::new(0.0, 1.0),
            ),
            1 => (
                Pos2::new(0.0, 1.0),
                Pos2::new(0.0, 0.0),
                Pos2::new(1.0, 0.0),
                Pos2::new(1.0, 1.0),
            ),
            2 => (
                Pos2::new(1.0, 1.0),
                Pos2::new(0.0, 1.0),
                Pos2::new(0.0, 0.0),
                Pos2::new(1.0, 0.0),
            ),
            3 => (
                Pos2::new(1.0, 0.0),
                Pos2::new(1.0, 1.0),
                Pos2::new(0.0, 1.0),
                Pos2::new(0.0, 0.0),
            ),
            _ => (
                Pos2::new(0.0, 0.0),
                Pos2::new(1.0, 0.0),
                Pos2::new(1.0, 1.0),
                Pos2::new(0.0, 1.0),
            ),
        };

        let mut mesh = Mesh::with_texture(tex.id());
        let tint = Color32::WHITE;

        // Vertices: Top-Left, Top-Right, Bottom-Right, Bottom-Left
        let idx_tl = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex {
            pos: dst_rect.left_top(),
            uv: uv_tl,
            color: tint,
        });
        mesh.vertices.push(Vertex {
            pos: dst_rect.right_top(),
            uv: uv_tr,
            color: tint,
        });
        mesh.vertices.push(Vertex {
            pos: dst_rect.right_bottom(),
            uv: uv_br,
            color: tint,
        });
        mesh.vertices.push(Vertex {
            pos: dst_rect.left_bottom(),
            uv: uv_bl,
            color: tint,
        });

        // Two triangles forming the quad
        mesh.indices.extend_from_slice(&[
            idx_tl,
            idx_tl + 1,
            idx_tl + 2,
            idx_tl,
            idx_tl + 2,
            idx_tl + 3,
        ]);

        painter.add(mesh);
    } else {
        // Interactive welcome card with prominent Open buttons
        let center = available_rect.center();
        let card_size = Vec2::new(440.0, 240.0);
        let card_rect = Rect::from_center_size(center, card_size);

        // Draw translucent card container
        painter.rect_filled(
            card_rect,
            14.0,
            Color32::from_black_alpha(200),
        );
        painter.rect_stroke(
            card_rect,
            14.0,
            Stroke::new(1.0_f32, Color32::from_white_alpha(35)),
        );

        // Allocate interactive UI inside the centered card
        ui.allocate_ui_at_rect(card_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(22.0);

                ui.label(
                    egui::RichText::new("📷 Vizo Image Viewer")
                        .size(22.0)
                        .color(Color32::from_rgb(235, 235, 245))
                        .strong(),
                );

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("NEF • CR2/3 • SR2 • ARW • DNG • TIFF • JPEG • PNG")
                        .size(11.0)
                        .color(Color32::from_rgb(140, 145, 160)),
                );

                ui.add_space(24.0);

                // Prominent Open Buttons
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(14.0, 0.0);
                    // Center the two buttons
                    let total_btn_w = 160.0 + 160.0 + 14.0;
                    let pad = (ui.available_width() - total_btn_w).max(0.0) / 2.0;
                    ui.add_space(pad);

                    let open_file_btn = egui::Button::new(
                        egui::RichText::new("📂 Open Image")
                            .size(14.0)
                            .color(Color32::WHITE)
                            .strong(),
                    )
                    .min_size(Vec2::new(160.0, 38.0))
                    .fill(Color32::from_rgb(45, 85, 155))
                    .rounding(Rounding::same(8.0));

                    if ui
                        .add(open_file_btn)
                        .on_hover_text("Choose an image file (Ctrl + O or press O)")
                        .clicked()
                    {
                        triggered_action = Some(AppAction::OpenFile);
                    }

                    let open_folder_btn = egui::Button::new(
                        egui::RichText::new("📁 Open Folder")
                            .size(14.0)
                            .color(Color32::from_rgb(220, 225, 235)),
                    )
                    .min_size(Vec2::new(160.0, 38.0))
                    .fill(Color32::from_rgb(40, 48, 65))
                    .rounding(Rounding::same(8.0));

                    if ui
                        .add(open_folder_btn)
                        .on_hover_text("Choose a folder to view all images (Ctrl + Shift + O)")
                        .clicked()
                    {
                        triggered_action = Some(AppAction::OpenFolder);
                    }
                });

                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("or drag and drop an image anywhere")
                        .size(12.0)
                        .color(Color32::from_rgb(105, 110, 125)),
                );
            });
        });
    }

    (response, triggered_action)
}
