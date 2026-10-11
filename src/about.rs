//! About dialog modal.
//!
//! Displays application metadata including name, version, GitHub repository,
//! and copyright information.

use eframe::egui::{self, Color32, Frame, Margin, Rect, Rounding, Stroke, Vec2};

/// Renders the About dialog modal as a centered, elegant overlay.
pub fn render_about_dialog(ctx: &egui::Context, open: &mut bool) {
    if !*open {
        return;
    }

    let screen_rect = ctx.screen_rect();
    let modal_size = Vec2::new(380.0, 260.0);
    let modal_rect = Rect::from_center_size(screen_rect.center(), modal_size);

    // Dim background overlay
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("about_backdrop"),
    ));
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));

    egui::Area::new(egui::Id::new("about_dialog_area"))
        .order(egui::Order::Foreground)
        .fixed_pos(modal_rect.min)
        .show(ctx, |ui| {
            Frame::none()
                .fill(Color32::from_rgb(26, 27, 34))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(50)))
                .rounding(Rounding::same(12.0_f32))
                .inner_margin(Margin::same(24.0_f32))
                .show(ui, |ui| {
                    ui.set_width(modal_size.x - 48.0);
                    ui.vertical_centered(|ui| {
                        // Title & Icon
                        ui.label(
                            egui::RichText::new("📷 Vizo")
                                .size(24.0)
                                .color(Color32::from_rgb(240, 240, 250))
                                .strong(),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("High Performance Image & RAW Viewer")
                                .size(12.0)
                                .color(Color32::from_rgb(160, 165, 180)),
                        );

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Version info
                        let version = env!("CARGO_PKG_VERSION");
                        ui.label(
                            egui::RichText::new(format!("Version: {}", version))
                                .size(13.0)
                                .color(Color32::from_rgb(200, 205, 220)),
                        );

                        ui.add_space(6.0);

                        // GitHub repository link
                        let github_url = "https://github.com/altaykirecci/vizo";
                        ui.hyperlink_to(
                            egui::RichText::new(format!("GitHub: {}", github_url))
                                .size(12.0)
                                .color(Color32::from_rgb(100, 160, 240)),
                            github_url,
                        );

                        ui.add_space(10.0);

                        // Copyright info
                        ui.label(
                            egui::RichText::new("Altay Kireççi (c)(p)10112026")
                                .size(12.0)
                                .color(Color32::from_rgb(230, 210, 140))
                                .strong(),
                        );

                        ui.add_space(18.0);

                        // Close button
                        let close_btn = egui::Button::new(
                            egui::RichText::new("Kapat / Close")
                                .size(13.0)
                                .color(Color32::WHITE),
                        )
                        .min_size(Vec2::new(100.0, 30.0))
                        .fill(Color32::from_rgb(50, 60, 80))
                        .rounding(Rounding::same(6.0_f32));

                        if ui.add(close_btn).clicked() {
                            *open = false;
                        }
                    });
                });
        });
}
