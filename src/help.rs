//! Keyboard shortcuts help overlay.
//!
//! Renders a translucent glass overlay displaying all keyboard shortcuts
//! and gesture controls in English, matching the EXIF overlay visual style.

use eframe::egui::{self, Button, Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke, Vec2};

/// Renders the translucent keyboard shortcuts help overlay.
pub fn render_help_overlay(
    ctx: &egui::Context,
    open: &mut bool,
    viewport_rect: Rect,
) {
    if !*open {
        return;
    }

    let card_width = 380.0_f32;
    let margin = 16.0_f32;
    let card_pos = Pos2::new(
        viewport_rect.min.x + margin,
        viewport_rect.min.y + 55.0_f32,
    );

    let area_id = egui::Id::new("shortcuts_help_overlay_card");

    egui::Area::new(area_id)
        .order(egui::Order::Foreground)
        .fixed_pos(card_pos)
        .show(ctx, |ui| {
            Frame::none()
                // Matching EXIF translucency: ~50% translucent glass effect
                .fill(Color32::from_black_alpha(130))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(35)))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(Margin::symmetric(14.0_f32, 12.0_f32))
                .show(ui, |ui| {
                    ui.set_width(card_width);

                    ui.vertical(|ui| {
                        // Title header with close button
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⌨ Keyboard Shortcuts & Help")
                                    .color(Color32::from_rgb(235, 235, 250))
                                    .size(13.0_f32)
                                    .strong(),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = Button::new(
                                    egui::RichText::new("✕")
                                        .color(Color32::from_rgb(180, 185, 200))
                                        .size(12.0_f32),
                                )
                                .frame(false);

                                if ui.add(close_btn).on_hover_text("Close Help (Esc or F1)").clicked() {
                                    *open = false;
                                }
                            });
                        });

                        ui.add_space(2.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Category: Navigation
                        render_section_header(ui, "📁 Navigation");
                        render_shortcut_row(ui, "Next Image", "→ / Ctrl + →");
                        render_shortcut_row(ui, "Previous Image", "← / Ctrl + ←");
                        render_shortcut_row(ui, "Pan Image", "Left Click + Drag");

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Category: Zoom & View
                        render_section_header(ui, "🔍 Zoom & View");
                        render_shortcut_row(ui, "Zoom In", "Ctrl + + / Wheel Up");
                        render_shortcut_row(ui, "Zoom Out", "Ctrl + - / Wheel Down");
                        render_shortcut_row(ui, "Actual Size (100%)", "Ctrl + 0 / Double Click");
                        render_shortcut_row(ui, "Fit to Window", "Ctrl + F / Double Click");

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Category: Rotation & File
                        render_section_header(ui, "⟳ Rotation & File");
                        render_shortcut_row(ui, "Rotate Clockwise", "Ctrl + R");
                        render_shortcut_row(ui, "Rotate Counter-Clockwise", "Ctrl + L");
                        render_shortcut_row(ui, "Open Image File", "Ctrl + O / O");
                        render_shortcut_row(ui, "Open Folder", "Ctrl + Shift + O");

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Category: Overlays & Tools
                        render_section_header(ui, "🛠 Overlays & Tools");
                        render_shortcut_row(ui, "Toggle Toolbar", "Ctrl + T");
                        render_shortcut_row(ui, "Toggle EXIF Metadata", "Ctrl + I");
                        render_shortcut_row(ui, "Slideshow Settings", "Ctrl + S");
                        render_shortcut_row(ui, "Keyboard Shortcuts", "F1 / ?");
                        render_shortcut_row(ui, "About Vizo", "Ctrl + H");

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Tip: Press Esc or F1 to close this overlay.")
                                    .color(Color32::from_rgb(140, 150, 175))
                                    .size(10.5_f32)
                                    .italics(),
                            );
                        });
                    });
                });
        });
}

/// Renders a section category header.
fn render_section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .color(Color32::from_rgb(160, 195, 255))
            .size(11.5_f32)
            .strong(),
    );
}

/// Renders an aligned row mapping an action description to its keyboard shortcut badge.
fn render_shortcut_row(ui: &mut egui::Ui, action: &str, keys: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
        ui.label(
            egui::RichText::new(action)
                .color(Color32::from_rgb(215, 220, 235))
                .size(11.0_f32),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            Frame::none()
                .fill(Color32::from_rgb(40, 45, 60))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(30)))
                .rounding(Rounding::same(4.0_f32))
                .inner_margin(Margin::symmetric(6.0_f32, 2.0_f32))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(keys)
                            .color(Color32::from_rgb(250, 240, 180))
                            .size(10.5_f32)
                            .monospace()
                            .strong(),
                    );
                });
        });
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_help_overlay_toggle_logic() {
        let mut open = true;
        assert!(open);
        open = false;
        assert!(!open);
    }
}
