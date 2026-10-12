//! Application state and orchestration.
//!
//! Ties together navigation, decoding, transformations, keyboard shortcuts,
//! floating toolbar HUD, and EXIF metadata overlay into an `eframe::App`.

use crate::about::render_about_dialog;
use crate::decoder::decode_image;
use crate::error::{ViewerError, ViewerResult};
use crate::exif_info::{extract_exif, render_exif_overlay, ExifData};
use crate::help::render_help_overlay;
use crate::navigation::NavigationState;
use crate::shortcuts::{process_shortcuts, AppAction};
use crate::slideshow::{
    render_slideshow_dialog, render_slideshow_toast, SlideshowEvent, SlideshowState,
};
use crate::toolbar::{render_toolbar, ToolbarProps};
use crate::transform::ViewTransform;
use crate::viewer::render_canvas;
use eframe::egui::{
    self, CentralPanel, Context, TextureHandle, TextureOptions, Vec2, ViewportCommand,
};
use std::path::{Path, PathBuf};

/// Primary application struct managing all runtime state and subsystems.
pub struct VizoViewer {
    /// File discovery and directory navigation state.
    pub navigation: NavigationState,
    /// 2D pan, zoom, and 90-degree quadrant rotation transform.
    pub transform: ViewTransform,
    /// Whether the floating HUD toolbar is currently visible (Ctrl + T).
    pub show_toolbar: bool,
    /// Preserved toolbar visibility state to restore upon exiting slideshow.
    pub prev_toolbar_visible: bool,
    /// Whether the EXIF metadata card is currently visible (Ctrl + I).
    pub show_exif: bool,
    /// Whether the shortcuts help card is currently visible (F1 or ?).
    pub show_help: bool,
    /// Whether the About (Hakkında) dialog is open (Ctrl + H).
    pub show_about: bool,
    /// Slideshow controller and configuration state.
    pub slideshow: SlideshowState,
    /// Active GPU texture handle of the currently displayed image.
    pub current_texture: Option<TextureHandle>,
    /// Metadata and EXIF information of the currently active image.
    pub current_exif: Option<ExifData>,
    /// Last error encountered during file reading or decoding, if any.
    pub last_error: Option<ViewerError>,
    /// Whether the first frame has triggered auto-fitting.
    pub needs_initial_fit: bool,
}

impl VizoViewer {
    /// Initializes a new image viewer instance, optionally loading an initial image or folder.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        initial_file: Option<PathBuf>,
    ) -> Self {
        let mut app = Self {
            navigation: NavigationState::new(),
            transform: ViewTransform::new(),
            show_toolbar: true,
            prev_toolbar_visible: true,
            show_exif: false,
            show_help: false,
            show_about: false,
            slideshow: SlideshowState::new(),
            current_texture: None,
            current_exif: None,
            last_error: None,
            needs_initial_fit: true,
        };

        let start_file = initial_file.or_else(crate::macos_ext::take_pending_file);
        if let Some(path) = start_file {
            let _ = app.open_path(&path);
        }

        app
    }

    /// Opens a file or directory path, discovers sibling images, and extracts EXIF metadata.
    pub fn open_path(&mut self, path: &Path) -> ViewerResult<()> {
        self.last_error = None;
        let nav = if path.is_dir() {
            NavigationState::from_directory(path)?
        } else {
            NavigationState::from_file(path)?
        };
        self.navigation = nav;
        self.current_texture = None;
        self.needs_initial_fit = true;

        if let Some(target_file) = self.navigation.current_file().cloned() {
            match extract_exif(&target_file) {
                Ok(exif) => self.current_exif = Some(exif),
                Err(e) => {
                    self.current_exif = Some(ExifData {
                        filename: target_file
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        folder: target_file
                            .parent()
                            .unwrap_or(&target_file)
                            .to_string_lossy()
                            .to_string(),
                        ..Default::default()
                    });
                    eprintln!("EXIF extraction notice: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Legacy convenience wrapper around `open_path`.
    pub fn open_file(&mut self, path: &Path) -> ViewerResult<()> {
        self.open_path(path)
    }

    /// Reloads and decodes the image at the current navigation index into an `egui::TextureHandle`.
    fn load_texture_if_needed(&mut self, ctx: &Context, viewport_size: Vec2) {
        if self.current_texture.is_some() || self.navigation.total_images() == 0 {
            return;
        }

        let Some(path) = self.navigation.current_file().cloned() else {
            return;
        };

        match decode_image(&path) {
            Ok(decoded) => {
                let color_img = decoded.to_color_image();
                let img_size = Vec2::new(decoded.width as f32, decoded.height as f32);

                // Synchronize EXIF dimensions with true decoded image resolution
                if let Some(ref mut exif) = self.current_exif {
                    exif.width = Some(decoded.width as u32);
                    exif.height = Some(decoded.height as u32);
                    let mp = (decoded.width as f64 * decoded.height as f64) / 1_000_000.0;
                    exif.megapixels = Some(format!("{:.1} MP", mp));
                }

                let texture = ctx.load_texture(
                    "active_image_texture",
                    color_img,
                    TextureOptions::LINEAR,
                );
                self.current_texture = Some(texture);
                self.last_error = None;

                if self.needs_initial_fit && viewport_size.x > 0.0 && viewport_size.y > 0.0 {
                    self.transform.fit_to_viewport(img_size, viewport_size);
                    self.needs_initial_fit = false;
                }
            }
            Err(e) => {
                self.last_error = Some(e);
                self.current_texture = None;
            }
        }
    }

    /// Starts automated slideshow playback, auto-closing modals, hiding toolbar, and entering fullscreen.
    pub fn start_slideshow(&mut self, ctx: &Context) {
        self.prev_toolbar_visible = self.show_toolbar;
        self.show_toolbar = false;
        self.show_exif = false;
        self.show_help = false;
        self.show_about = false;
        self.slideshow.start();
        // Cross-platform fullscreen via egui ViewportCommand
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
        ctx.request_repaint();
    }

    /// Stops automated slideshow playback, restoring previous toolbar visibility and exiting fullscreen.
    pub fn stop_slideshow(&mut self, ctx: &Context) {
        self.slideshow.stop();
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
        self.show_toolbar = self.prev_toolbar_visible;
        ctx.request_repaint();
    }

    /// Executes the specified action triggered by shortcut or toolbar control.
    pub fn handle_action(&mut self, action: AppAction, ctx: &Context) {
        match action {
            AppAction::ToggleToolbar => {
                self.show_toolbar = !self.show_toolbar;
            }
            AppAction::ToggleExif => {
                self.show_exif = !self.show_exif;
            }
            AppAction::ToggleAbout => {
                self.show_about = !self.show_about;
                ctx.request_repaint();
            }
            AppAction::ToggleHelp => {
                self.show_help = !self.show_help;
                ctx.request_repaint();
            }
            AppAction::ToggleSlideshow => {
                if self.slideshow.is_playing {
                    self.stop_slideshow(ctx);
                } else {
                    self.slideshow.toggle_dialog();
                    ctx.request_repaint();
                }
            }
            AppAction::CloseOverlays => {
                if self.slideshow.is_playing {
                    self.stop_slideshow(ctx);
                } else if self.slideshow.is_dialog_open {
                    self.slideshow.is_dialog_open = false;
                } else if self.show_help {
                    self.show_help = false;
                } else if self.show_about {
                    self.show_about = false;
                } else if self.show_exif {
                    self.show_exif = false;
                }
                ctx.request_repaint();
            }
            AppAction::NextImage => {
                if let Some(next_path) = self.navigation.next().cloned() {
                    self.current_texture = None;
                    self.last_error = None;
                    let _ = self.update_exif(&next_path);
                    self.needs_initial_fit = true;
                    ctx.request_repaint();
                }
            }
            AppAction::PreviousImage => {
                if let Some(prev_path) = self.navigation.previous().cloned() {
                    self.current_texture = None;
                    self.last_error = None;
                    let _ = self.update_exif(&prev_path);
                    self.needs_initial_fit = true;
                    ctx.request_repaint();
                }
            }
            AppAction::RotateClockwise => {
                self.transform.rotate_clockwise();
                ctx.request_repaint();
            }
            AppAction::RotateCounterClockwise => {
                self.transform.rotate_counter_clockwise();
                ctx.request_repaint();
            }
            AppAction::ZoomIn => {
                self.transform.zoom_in();
                ctx.request_repaint();
            }
            AppAction::ZoomOut => {
                self.transform.zoom_out();
                ctx.request_repaint();
            }
            AppAction::ActualSize => {
                self.transform.actual_size();
                ctx.request_repaint();
            }
            AppAction::FitToScreen => {
                if let Some(ref tex) = self.current_texture {
                    let screen_size = ctx.screen_rect().size();
                    self.transform.fit_to_viewport(tex.size_vec2(), screen_size);
                }
                ctx.request_repaint();
            }
            AppAction::OpenFile => {
                self.open_file_dialog();
                ctx.request_repaint();
            }
            AppAction::OpenFolder => {
                self.open_folder_dialog();
                ctx.request_repaint();
            }
        }
    }

    /// Opens a native file dialog to select an image file.
    pub fn open_file_dialog(&mut self) {
        let file = rfd::FileDialog::new()
            .set_title("Open Image File")
            .add_filter(
                "Supported Images & RAW",
                &[
                    "jpg", "jpeg", "png", "webp", "tif", "tiff", "bmp", "gif", "ico", "tga",
                    "avif", "qoi", "hdr", "exr", "heic", "heif", "nef", "cr2", "cr3", "crw",
                    "sr2", "arw", "dng", "orf", "rw2", "pef", "raf",
                ],
            )
            .add_filter("All Files", &["*"])
            .pick_file();

        if let Some(picked) = file {
            if let Err(e) = self.open_path(&picked) {
                self.last_error = Some(e);
            }
        }
    }

    /// Opens a native directory dialog to select an image folder.
    pub fn open_folder_dialog(&mut self) {
        let folder = rfd::FileDialog::new()
            .set_title("Open Image Folder")
            .pick_folder();

        if let Some(picked) = folder {
            if let Err(e) = self.open_path(&picked) {
                self.last_error = Some(e);
            }
        }
    }

    /// Updates EXIF data for the active image file.
    fn update_exif(&mut self, path: &Path) -> ViewerResult<()> {
        match extract_exif(path) {
            Ok(data) => {
                self.current_exif = Some(data);
                Ok(())
            }
            Err(e) => {
                self.current_exif = Some(ExifData {
                    filename: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    folder: path.parent().unwrap_or(path).to_string_lossy().to_string(),
                    ..Default::default()
                });
                Err(e)
            }
        }
    }
}

impl eframe::App for VizoViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle drag and drop files from OS
        ctx.input(|i| {
            if let Some(dropped) = i.raw.dropped_files.first() {
                if let Some(ref path) = dropped.path {
                    let _ = self.open_path(path);
                }
            }
        });

        // Handle files passed via macOS Finder / Open With / LaunchServices
        while let Some(path) = crate::macos_ext::take_pending_file() {
            let _ = self.open_path(&path);
            ctx.request_repaint();
        }

        // Automated slideshow progression
        if self.slideshow.is_playing {
            let now = std::time::Instant::now();
            let interval = std::time::Duration::from_secs(self.slideshow.interval_seconds as u64);
            let should_advance = match self.slideshow.last_advance {
                Some(last) => now.duration_since(last) >= interval,
                None => true,
            };

            if should_advance {
                self.slideshow.last_advance = Some(now);
                if let Some(next_path) = self.navigation.next().cloned() {
                    self.current_texture = None;
                    self.last_error = None;
                    let _ = self.update_exif(&next_path);
                    self.needs_initial_fit = true;
                    ctx.request_repaint();
                }
            }

            let remaining = match self.slideshow.last_advance {
                Some(last) => interval.saturating_sub(now.duration_since(last)),
                None => std::time::Duration::from_millis(100),
            };
            ctx.request_repaint_after(remaining);
        }

        // 1. Process all keyboard shortcuts
        let actions = process_shortcuts(ctx);
        for action in actions {
            self.handle_action(action, ctx);
        }

        // 2. Render primary central panel canvas
        CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let viewport_rect = ui.available_rect_before_wrap();
                self.load_texture_if_needed(ctx, viewport_rect.size());

                let (canvas_resp, canvas_action) = render_canvas(
                    ui,
                    self.current_texture.as_ref(),
                    &mut self.transform,
                    self.last_error.as_ref(),
                );

                if let Some(action) = canvas_action {
                    self.handle_action(action, ctx);
                }

                // Double click toggles between fit and 100%
                if canvas_resp.double_clicked() {
                    if (self.transform.zoom - 1.0).abs() < 0.05 {
                        self.handle_action(AppAction::FitToScreen, ctx);
                    } else {
                        self.handle_action(AppAction::ActualSize, ctx);
                    }
                }

                // 3. Render floating HUD toolbar if toggled on
                if self.show_toolbar {
                    let (curr_idx, total_count) = self.navigation.position_info();
                    let current_filename = self
                        .navigation
                        .current_file()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str());

                    let zoom_pct = (self.transform.zoom * 100.0).round() as u32;

                    let toolbar_props = ToolbarProps {
                        current_index: curr_idx,
                        total_images: total_count,
                        filename: current_filename,
                        zoom_percent: zoom_pct,
                        exif_active: self.show_exif,
                        slideshow_active: self.slideshow.is_playing,
                        help_active: self.show_help,
                    };

                    if let Some(action) = render_toolbar(ctx, viewport_rect, toolbar_props) {
                        self.handle_action(action, ctx);
                    }
                }

                // 4. Render EXIF metadata card if toggled on
                if self.show_exif {
                    if let Some(ref exif_data) = self.current_exif {
                        render_exif_overlay(ctx, exif_data, viewport_rect);
                    }
                }

                // 5. Render Shortcuts Help overlay card if toggled on
                if self.show_help {
                    render_help_overlay(ctx, &mut self.show_help, viewport_rect);
                }

                // 6. Render Slideshow settings dialog if opened
                if let Some(event) = render_slideshow_dialog(
                    ctx,
                    &mut self.slideshow,
                    self.navigation.total_images(),
                ) {
                    match event {
                        SlideshowEvent::Started => self.start_slideshow(ctx),
                        SlideshowEvent::Stopped => self.stop_slideshow(ctx),
                    }
                }

                // 7. Render bottom Esc hint toast during slideshow playback
                render_slideshow_toast(ctx, &self.slideshow, viewport_rect);

                // 8. Render About (Hakkında) modal dialog if toggled on
                render_about_dialog(ctx, &mut self.show_about);
            });
    }
}
