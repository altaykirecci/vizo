//! Keyboard shortcut processing.
//!
//! Centralizes event handling for all `Ctrl + <Key>` combinations,
//! providing fast navigation and complete keyboard controllability.

use eframe::egui::{Context, Key};

/// Actions triggered by keyboard shortcuts or UI controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    /// Toggle visibility of the floating HUD toolbar (Ctrl + T).
    ToggleToolbar,
    /// Toggle visibility of the EXIF details card (Ctrl + I).
    ToggleExif,
    /// Advance to the next image in the folder (Ctrl + Right or Right arrow).
    NextImage,
    /// Return to the previous image in the folder (Ctrl + Left or Left arrow).
    PreviousImage,
    /// Rotate image 90 degrees clockwise (Ctrl + R).
    RotateClockwise,
    /// Rotate image 90 degrees counter-clockwise (Ctrl + L).
    RotateCounterClockwise,
    /// Magnify image zoom level (Ctrl + Plus / Ctrl + =).
    ZoomIn,
    /// Reduce image zoom level (Ctrl + Minus).
    ZoomOut,
    /// Reset magnification to 100% native pixel resolution (Ctrl + 0).
    ActualSize,
    /// Adjust zoom so the image fits fully into the window (Ctrl + F).
    FitToScreen,
    /// Open a native file dialog to pick an image file (Ctrl + O or O).
    OpenFile,
    /// Open a native folder dialog to pick an entire image folder (Ctrl + Shift + O).
    OpenFolder,
    /// Toggle visibility of the About dialog (Ctrl + H or F1).
    ToggleAbout,
}

/// Polls keyboard input events from `egui::Context` and extracts matched shortcut actions.
pub fn process_shortcuts(ctx: &Context) -> Vec<AppAction> {
    let mut actions = Vec::new();

    ctx.input(|i| {
        let ctrl = i.modifiers.command || i.modifiers.ctrl;
        let shift = i.modifiers.shift;

        // F1 help shortcut
        if i.key_pressed(Key::F1) {
            actions.push(AppAction::ToggleAbout);
        }

        // Navigation shortcuts (Support both Ctrl+Arrow and raw Arrow for ergonomics)
        if (ctrl && i.key_pressed(Key::ArrowRight)) || i.key_pressed(Key::ArrowRight) {
            actions.push(AppAction::NextImage);
        }
        if (ctrl && i.key_pressed(Key::ArrowLeft)) || i.key_pressed(Key::ArrowLeft) {
            actions.push(AppAction::PreviousImage);
        }

        // Dedicated Ctrl + <Key> shortcuts
        if ctrl {
            if i.key_pressed(Key::T) {
                actions.push(AppAction::ToggleToolbar);
            }
            if i.key_pressed(Key::I) {
                actions.push(AppAction::ToggleExif);
            }
            if i.key_pressed(Key::H) {
                actions.push(AppAction::ToggleAbout);
            }
            if i.key_pressed(Key::R) {
                actions.push(AppAction::RotateClockwise);
            }
            if i.key_pressed(Key::L) {
                actions.push(AppAction::RotateCounterClockwise);
            }
            if i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals) {
                actions.push(AppAction::ZoomIn);
            }
            if i.key_pressed(Key::Minus) {
                actions.push(AppAction::ZoomOut);
            }
            if i.key_pressed(Key::Num0) {
                actions.push(AppAction::ActualSize);
            }
            if i.key_pressed(Key::F) {
                actions.push(AppAction::FitToScreen);
            }
            // File & folder open shortcuts
            if shift && i.key_pressed(Key::O) {
                actions.push(AppAction::OpenFolder);
            } else if i.key_pressed(Key::O) {
                actions.push(AppAction::OpenFile);
            }
        } else if !i.modifiers.alt && i.key_pressed(Key::O) {
            // Direct 'O' key without Ctrl for ultra-fast opening when viewer is focused
            actions.push(AppAction::OpenFile);
        }
    });

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_action_variants() {
        assert_ne!(AppAction::OpenFile, AppAction::OpenFolder);
        assert_ne!(AppAction::ToggleToolbar, AppAction::ToggleExif);
        assert_ne!(AppAction::ZoomIn, AppAction::ZoomOut);
    }
}
