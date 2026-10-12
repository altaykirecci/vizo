//! Slideshow controller, configuration dialog, and fullscreen status toast.
//!
//! Provides automated photo advancement with user-configurable transition
//! intervals between 3 and 60 seconds, equipped with stepper input controls,
//! auto-closing modals, and transient bottom notification toasts.

use eframe::egui::{
    self, Button, Color32, DragValue, Frame, Margin, Pos2, Rect, Rounding, Stroke, Vec2,
};
use std::time::{Duration, Instant};

/// Events emitted by slideshow dialog interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideshowEvent {
    /// User initiated slideshow playback.
    Started,
    /// User stopped slideshow playback.
    Stopped,
}

/// Runtime state for the slideshow subsystem.
#[derive(Debug, Clone)]
pub struct SlideshowState {
    /// Whether the slideshow configuration popup dialog is open.
    pub is_dialog_open: bool,
    /// Whether automatic slideshow cycling is currently running.
    pub is_playing: bool,
    /// Duration in seconds each image is shown before advancing (3..=60).
    pub interval_seconds: u32,
    /// Timestamp when the current image was displayed or last advanced.
    pub last_advance: Option<Instant>,
    /// Expiration time for displaying the transient bottom "Press Esc to exit" banner.
    pub toast_until: Option<Instant>,
}

impl Default for SlideshowState {
    fn default() -> Self {
        Self {
            is_dialog_open: false,
            is_playing: false,
            interval_seconds: 5,
            last_advance: None,
            toast_until: None,
        }
    }
}

impl SlideshowState {
    /// Minimum allowed slideshow interval in seconds.
    pub const MIN_INTERVAL_SECS: u32 = 3;
    /// Maximum allowed slideshow interval in seconds.
    pub const MAX_INTERVAL_SECS: u32 = 60;
    /// Duration to display the exit hint banner after starting slideshow.
    pub const TOAST_DURATION_SECS: u64 = 4;

    /// Creates a new default slideshow state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles the configuration modal dialog.
    pub fn toggle_dialog(&mut self) {
        self.is_dialog_open = !self.is_dialog_open;
    }

    /// Starts or resumes slideshow playback, auto-closing the popup and setting toast timer.
    pub fn start(&mut self) {
        self.interval_seconds = self
            .interval_seconds
            .clamp(Self::MIN_INTERVAL_SECS, Self::MAX_INTERVAL_SECS);
        self.is_playing = true;
        self.is_dialog_open = false; // Automatically close popup modal
        self.last_advance = Some(Instant::now());
        self.toast_until = Some(Instant::now() + Duration::from_secs(Self::TOAST_DURATION_SECS));
    }

    /// Pauses or stops slideshow playback.
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.last_advance = None;
        self.toast_until = None;
    }

    /// Toggles play/pause status.
    pub fn toggle_playback(&mut self) {
        if self.is_playing {
            self.stop();
        } else {
            self.start();
        }
    }

    /// Increments interval by 1 second, respecting the 60-second limit.
    pub fn increment_interval(&mut self) {
        if self.interval_seconds < Self::MAX_INTERVAL_SECS {
            self.interval_seconds += 1;
        }
    }

    /// Decrements interval by 1 second, respecting the 3-second limit.
    pub fn decrement_interval(&mut self) {
        if self.interval_seconds > Self::MIN_INTERVAL_SECS {
            self.interval_seconds -= 1;
        }
    }

    /// Sets interval safely within bounds [3, 60].
    pub fn set_interval(&mut self, seconds: u32) {
        self.interval_seconds = seconds.clamp(Self::MIN_INTERVAL_SECS, Self::MAX_INTERVAL_SECS);
    }
}

/// Renders the slideshow configuration popup modal.
///
/// Returns `Some(SlideshowEvent)` when a play/stop button triggers a state change.
pub fn render_slideshow_dialog(
    ctx: &egui::Context,
    state: &mut SlideshowState,
    total_images: usize,
) -> Option<SlideshowEvent> {
    if !state.is_dialog_open {
        return None;
    }

    let mut event = None;
    let screen_rect = ctx.screen_rect();
    let modal_size = Vec2::new(390.0, 270.0);
    let modal_rect = Rect::from_center_size(screen_rect.center(), modal_size);

    // Dim background overlay
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("slideshow_backdrop"),
    ));
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(130));

    egui::Area::new(egui::Id::new("slideshow_dialog_area"))
        .order(egui::Order::Foreground)
        .fixed_pos(modal_rect.min)
        .show(ctx, |ui| {
            Frame::none()
                .fill(Color32::from_rgb(26, 27, 34))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(50)))
                .rounding(Rounding::same(12.0_f32))
                .inner_margin(Margin::same(20.0_f32))
                .show(ui, |ui| {
                    ui.set_width(modal_size.x - 40.0);

                    // Header row
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⏱ Slideshow Settings")
                                .size(17.0)
                                .color(Color32::from_rgb(240, 240, 255))
                                .strong(),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = Button::new(
                                egui::RichText::new("✕")
                                    .color(Color32::from_rgb(180, 185, 200))
                                    .size(13.0),
                            )
                            .frame(false);

                            if ui.add(close_btn).on_hover_text("Close (Esc)").clicked() {
                                state.is_dialog_open = false;
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Status and photo count
                    ui.horizontal(|ui| {
                        let count_text = if total_images > 0 {
                            format!("📁 Available Photos: {} files", total_images)
                        } else {
                            "📁 No photos currently loaded".to_string()
                        };
                        ui.label(
                            egui::RichText::new(count_text)
                                .size(12.0)
                                .color(Color32::from_rgb(170, 180, 205)),
                        );
                    });

                    ui.add_space(10.0);

                    // Interval selector section with stepper buttons
                    ui.label(
                        egui::RichText::new("Display duration per photo:")
                            .size(12.5)
                            .color(Color32::from_rgb(220, 225, 240)),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

                        // Decrease button
                        let dec_btn = Button::new(
                            egui::RichText::new("➖")
                                .color(Color32::from_rgb(220, 220, 235))
                                .size(13.0),
                        )
                        .min_size(Vec2::new(34.0, 30.0))
                        .fill(Color32::from_rgb(45, 52, 70))
                        .rounding(Rounding::same(6.0));

                        if ui
                            .add_enabled(
                                state.interval_seconds > SlideshowState::MIN_INTERVAL_SECS,
                                dec_btn,
                            )
                            .on_hover_text("Decrease 1 second")
                            .clicked()
                        {
                            state.decrement_interval();
                        }

                        // Increase/decrease editable input box (DragValue)
                        let mut interval = state.interval_seconds;
                        let drag_resp = ui.add(
                            DragValue::new(&mut interval)
                                .range(SlideshowState::MIN_INTERVAL_SECS..=SlideshowState::MAX_INTERVAL_SECS)
                                .speed(0.2)
                                .suffix(" seconds"),
                        );
                        if drag_resp.changed() {
                            state.set_interval(interval);
                        }

                        // Increase button
                        let inc_btn = Button::new(
                            egui::RichText::new("➕")
                                .color(Color32::from_rgb(220, 220, 235))
                                .size(13.0),
                        )
                        .min_size(Vec2::new(34.0, 30.0))
                        .fill(Color32::from_rgb(45, 52, 70))
                        .rounding(Rounding::same(6.0));

                        if ui
                            .add_enabled(
                                state.interval_seconds < SlideshowState::MAX_INTERVAL_SECS,
                                inc_btn,
                            )
                            .on_hover_text("Increase 1 second")
                            .clicked()
                        {
                            state.increment_interval();
                        }

                        // Bounds indicator
                        ui.label(
                            egui::RichText::new("(3s - 60s)")
                                .size(11.0)
                                .color(Color32::from_rgb(140, 150, 170)),
                        );
                    });

                    ui.add_space(8.0);

                    // Quick duration presets
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                        ui.label(
                            egui::RichText::new("Quick:")
                                .size(11.0)
                                .color(Color32::from_rgb(150, 160, 180)),
                        );

                        for &preset in &[3, 5, 10, 15, 30] {
                            let is_current = state.interval_seconds == preset;
                            let preset_btn = Button::new(
                                egui::RichText::new(format!("{}s", preset))
                                    .size(11.0)
                                    .color(if is_current {
                                        Color32::from_rgb(255, 230, 130)
                                    } else {
                                        Color32::from_rgb(190, 195, 215)
                                    }),
                            )
                            .fill(if is_current {
                                Color32::from_rgb(60, 75, 110)
                            } else {
                                Color32::from_rgb(38, 44, 58)
                            })
                            .rounding(Rounding::same(4.0));

                            if ui.add(preset_btn).clicked() {
                                state.set_interval(preset);
                            }
                        }
                    });

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Controls: Play / Stop and Close buttons
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                        if state.is_playing {
                            let stop_btn = Button::new(
                                egui::RichText::new("⏹ Stop Slideshow")
                                    .size(13.0)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .min_size(Vec2::new(145.0, 32.0))
                            .fill(Color32::from_rgb(175, 55, 65))
                            .rounding(Rounding::same(6.0));

                            if ui.add(stop_btn).clicked() {
                                state.stop();
                                event = Some(SlideshowEvent::Stopped);
                            }
                        } else {
                            let can_start = total_images > 0;
                            let start_btn = Button::new(
                                egui::RichText::new("▶ Start Slideshow")
                                    .size(13.0)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .min_size(Vec2::new(145.0, 32.0))
                            .fill(Color32::from_rgb(45, 130, 75))
                            .rounding(Rounding::same(6.0));

                            if ui.add_enabled(can_start, start_btn).clicked() {
                                state.start();
                                event = Some(SlideshowEvent::Started);
                            }
                        }

                        // Close button
                        let dismiss_btn = Button::new(
                            egui::RichText::new("Close")
                                .size(13.0)
                                .color(Color32::from_rgb(210, 215, 230)),
                        )
                        .min_size(Vec2::new(80.0, 32.0))
                        .fill(Color32::from_rgb(45, 52, 68))
                        .rounding(Rounding::same(6.0));

                        if ui.add(dismiss_btn).clicked() {
                            state.is_dialog_open = false;
                        }

                        // Playback status label
                        if state.is_playing {
                            ui.label(
                                egui::RichText::new("● Playing")
                                    .color(Color32::from_rgb(100, 220, 130))
                                    .size(12.0)
                                    .strong(),
                            );
                        }
                    });
                });
        });

    event
}

/// Renders a transient bottom translucent banner notifying the user that Esc stops the slideshow.
pub fn render_slideshow_toast(
    ctx: &egui::Context,
    state: &SlideshowState,
    viewport_rect: Rect,
) {
    if !state.is_playing {
        return;
    }

    if let Some(until) = state.toast_until {
        let now = Instant::now();
        if now < until {
            let remaining = until.duration_since(now);
            // Request repaint precisely when the banner expires so it cleanly fades away
            ctx.request_repaint_after(remaining);

            let toast_pos = Pos2::new(
                viewport_rect.center().x,
                viewport_rect.max.y - 38.0_f32,
            );

            egui::Area::new(egui::Id::new("slideshow_esc_hint_toast"))
                .order(egui::Order::Foreground)
                .fixed_pos(toast_pos)
                .pivot(egui::Align2::CENTER_BOTTOM)
                .show(ctx, |ui| {
                    Frame::none()
                        // Translucent glass badge
                        .fill(Color32::from_black_alpha(150))
                        .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(45)))
                        .rounding(Rounding::same(20.0_f32))
                        .inner_margin(Margin::symmetric(20.0_f32, 8.0_f32))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                                ui.label(
                                    egui::RichText::new("⌨ Press Esc to exit slideshow (Sonlandırmak için Esc'ye basın)")
                                        .color(Color32::from_rgb(235, 240, 255))
                                        .size(12.0_f32)
                                        .strong(),
                                );
                            });
                        });
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slideshow_interval_bounds() {
        let mut state = SlideshowState::new();
        assert_eq!(state.interval_seconds, 5);

        // Clamping below 3
        state.set_interval(1);
        assert_eq!(state.interval_seconds, SlideshowState::MIN_INTERVAL_SECS);

        // Decrement cannot go below 3
        state.decrement_interval();
        assert_eq!(state.interval_seconds, SlideshowState::MIN_INTERVAL_SECS);

        // Clamping above 60
        state.set_interval(100);
        assert_eq!(state.interval_seconds, SlideshowState::MAX_INTERVAL_SECS);

        // Increment cannot exceed 60
        state.increment_interval();
        assert_eq!(state.interval_seconds, SlideshowState::MAX_INTERVAL_SECS);
    }

    #[test]
    fn test_slideshow_playback_state() {
        let mut state = SlideshowState::new();
        assert!(!state.is_playing);
        assert!(state.last_advance.is_none());
        assert!(state.toast_until.is_none());

        state.is_dialog_open = true;
        state.start();
        assert!(state.is_playing);
        // Modal must auto-close on start
        assert!(!state.is_dialog_open);
        assert!(state.last_advance.is_some());
        assert!(state.toast_until.is_some());

        state.stop();
        assert!(!state.is_playing);
        assert!(state.last_advance.is_none());
        assert!(state.toast_until.is_none());

        state.toggle_playback();
        assert!(state.is_playing);
    }

    #[test]
    fn test_slideshow_dialog_toggle() {
        let mut state = SlideshowState::new();
        assert!(!state.is_dialog_open);

        state.toggle_dialog();
        assert!(state.is_dialog_open);

        state.toggle_dialog();
        assert!(!state.is_dialog_open);
    }
}
