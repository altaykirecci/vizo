//! Floating HUD Toolbar rendering.
//!
//! Provides a modern, translucent floating toolbar displaying navigation,
//! rotation, zoom controls, and file indicators with shortcut tooltips.

use crate::shortcuts::AppAction;
use eframe::egui::{self, Button, Color32, Frame, Margin, Pos2, Rect, Rounding, Stroke, Vec2};

/// Parameters passed to render the toolbar HUD.
pub struct ToolbarProps<'a> {
    /// 1-based index of the current image.
    pub current_index: usize,
    /// Total image count in the directory.
    pub total_images: usize,
    /// Current image filename.
    pub filename: Option<&'a str>,
    /// Current zoom percentage (e.g. 100).
    pub zoom_percent: u32,
    /// Whether EXIF panel is currently shown.
    pub exif_active: bool,
}

/// Renders the floating toolbar HUD centered near the top edge of the viewport.
pub fn render_toolbar(
    ctx: &egui::Context,
    viewport_rect: Rect,
    props: ToolbarProps<'_>,
) -> Option<AppAction> {
    let mut triggered_action = None;

    let toolbar_y = viewport_rect.min.y + 16.0_f32;
    let toolbar_center_x = viewport_rect.center().x;

    let area_id = egui::Id::new("floating_hud_toolbar");

    egui::Area::new(area_id)
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::new(toolbar_center_x, toolbar_y))
        .pivot(egui::Align2::CENTER_TOP)
        .show(ctx, |ui| {
            Frame::none()
                .fill(Color32::from_black_alpha(225))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(38)))
                .rounding(Rounding::same(24.0_f32))
                .inner_margin(Margin::symmetric(14.0_f32, 7.0_f32))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(6.0_f32, 0.0_f32);

                        // File open button with clear label
                        let open_file_btn = Button::new(
                            egui::RichText::new("📂 Open")
                                .color(Color32::from_rgb(230, 230, 245))
                                .size(13.0_f32),
                        )
                        .fill(Color32::from_rgb(40, 50, 75))
                        .rounding(Rounding::same(12.0_f32));

                        if ui
                            .add(open_file_btn)
                            .on_hover_text("Open Image File (Ctrl + O or press O)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::OpenFile);
                        }

                        // Folder open button
                        let open_folder_btn = Button::new(
                            egui::RichText::new("📁 Folder")
                                .color(Color32::from_rgb(210, 210, 225))
                                .size(13.0_f32),
                        )
                        .fill(Color32::from_rgb(32, 38, 52))
                        .rounding(Rounding::same(12.0_f32));

                        if ui
                            .add(open_folder_btn)
                            .on_hover_text("Open Entire Image Folder (Ctrl + Shift + O)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::OpenFolder);
                        }

                        ui.separator();

                        // Navigation buttons
                        let prev_btn = ui
                            .add_enabled(props.total_images > 0, Button::new("◀").frame(false))
                            .on_hover_text("Previous Image (Ctrl + Left or Left Arrow)");
                        if prev_btn.clicked() {
                            triggered_action = Some(AppAction::PreviousImage);
                        }

                        let next_btn = ui
                            .add_enabled(props.total_images > 0, Button::new("▶").frame(false))
                            .on_hover_text("Next Image (Ctrl + Right or Right Arrow)");
                        if next_btn.clicked() {
                            triggered_action = Some(AppAction::NextImage);
                        }

                        ui.separator();

                        // Rotation buttons
                        if ui
                            .add(Button::new("⟲").frame(false))
                            .on_hover_text("Rotate 90° Counter-Clockwise (Ctrl + L)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::RotateCounterClockwise);
                        }

                        if ui
                            .add(Button::new("⟳").frame(false))
                            .on_hover_text("Rotate 90° Clockwise (Ctrl + R)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::RotateClockwise);
                        }

                        ui.separator();

                        // Zoom controls
                        if ui
                            .add(Button::new("➖").frame(false))
                            .on_hover_text("Zoom Out (Ctrl + -)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::ZoomOut);
                        }

                        let zoom_label = format!("{}%", props.zoom_percent);
                        if ui
                            .add(Button::new(zoom_label).frame(false))
                            .on_hover_text("Actual Size 100% (Ctrl + 0)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::ActualSize);
                        }

                        if ui
                            .add(Button::new("➕").frame(false))
                            .on_hover_text("Zoom In (Ctrl + +)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::ZoomIn);
                        }

                        if ui
                            .add(Button::new("⛶").frame(false))
                            .on_hover_text("Fit to Window (Ctrl + F)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::FitToScreen);
                        }

                        ui.separator();

                        // EXIF Metadata toggle button
                        let exif_btn = Button::new(if props.exif_active { "📷 EXIF [On]" } else { "📷 EXIF" })
                            .frame(false);
                        if ui
                            .add(exif_btn)
                            .on_hover_text("Toggle EXIF & Camera Metadata (Ctrl + I)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::ToggleExif);
                        }

                        // About (Hakkında) button with pure 'i' icon as requested
                        let about_btn = Button::new("ℹ")
                            .frame(false);
                        if ui
                            .add(about_btn)
                            .on_hover_text("About Vizo / Hakkında (Ctrl + H or F1)")
                            .clicked()
                        {
                            triggered_action = Some(AppAction::ToggleAbout);
                        }

                        // Position / Filename readout
                        if props.total_images > 0 {
                            ui.separator();
                            let count_text = format!(
                                "{} / {}",
                                props.current_index, props.total_images
                            );
                            ui.label(
                                egui::RichText::new(count_text)
                                    .color(Color32::from_rgb(180, 180, 190))
                                    .size(12.0_f32),
                            );

                            if let Some(name) = props.filename {
                                ui.label(
                                    egui::RichText::new(name)
                                        .color(Color32::from_rgb(220, 220, 230))
                                        .size(12.0_f32)
                                        .strong(),
                                );
                            }
                        }
                    });
                });
        });

    triggered_action
}
