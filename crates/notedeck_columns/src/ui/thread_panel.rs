use egui::{Color32, RichText, Sense, Vec2};
use notedeck::{tr, JobsCache, NoteAction, NoteContext};
use notedeck_ui::NoteOptions;

use crate::timeline::thread::Threads;
use crate::ui::ThreadView;

pub const THREAD_PANEL_WIDTH: f32 = 420.0;

pub struct ThreadPanel {
    pub is_open: bool,
    pub selected_thread_id: Option<[u8; 32]>,
}

pub enum ThreadPanelAction {
    Close,
}

impl ThreadPanel {
    pub fn new() -> Self {
        Self {
            is_open: false,
            selected_thread_id: None,
        }
    }

    pub fn open(&mut self, thread_id: [u8; 32]) {
        self.is_open = true;
        self.selected_thread_id = Some(thread_id);
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn show<'a, 'd>(
        &mut self,
        ui: &mut egui::Ui,
        threads: &'a mut Threads,
        note_options: NoteOptions,
        note_context: &'a mut NoteContext<'d>,
        jobs: &'a mut JobsCache,
        col: usize,
    ) -> (Option<ThreadPanelAction>, Option<NoteAction>) {
        if !self.is_open || self.selected_thread_id.is_none() {
            return (None, None);
        }

        let mut panel_action: Option<ThreadPanelAction> = None;
        let mut note_action: Option<NoteAction> = None;

        let screen_rect = ui.ctx().screen_rect();
        let panel_width = THREAD_PANEL_WIDTH;

        // Panel positioned at the right side of the screen
        let panel_rect = egui::Rect::from_min_size(
            egui::pos2(screen_rect.max.x - panel_width, screen_rect.min.y),
            Vec2::new(panel_width, screen_rect.height()),
        );

        // Semi-transparent overlay on the left (non-panel area)
        let overlay_rect = egui::Rect::from_min_size(
            egui::pos2(screen_rect.min.x, screen_rect.min.y),
            Vec2::new(screen_rect.width() - panel_width, screen_rect.height()),
        );

        // Draw overlay
        ui.painter().rect_filled(
            overlay_rect,
            0.0,
            Color32::from_black_alpha(100),
        );

        // Handle click on overlay to close
        let overlay_response = ui.interact(overlay_rect, egui::Id::new("thread_panel_overlay"), Sense::click());
        if overlay_response.clicked() {
            panel_action = Some(ThreadPanelAction::Close);
        }

        // Draw the panel
        egui::Area::new(egui::Id::new("thread_panel"))
            .fixed_pos(panel_rect.min)
            .movable(false)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                ui.set_clip_rect(panel_rect);

                // Panel background frame
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .stroke(egui::Stroke::new(
                        1.0,
                        if ui.visuals().dark_mode {
                            Color32::from_rgb(55, 65, 81)
                        } else {
                            Color32::from_rgb(229, 231, 235)
                        },
                    ))
                    .show(ui, |ui| {
                        ui.set_width(panel_width);
                        ui.set_height(screen_rect.height());

                        ui.vertical(|ui| {
                            // Header with close button
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);

                                ui.label(
                                    RichText::new(tr!(note_context.i18n, "Thread", "Thread panel header"))
                                        .size(16.0)
                                        .strong(),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(16.0);

                                    // Close button (X)
                                    let close_btn = ui.button(RichText::new("✕").size(20.0));

                                    if close_btn.clicked() {
                                        panel_action = Some(ThreadPanelAction::Close);
                                    }
                                });
                            });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            // Thread content
                            if let Some(thread_id) = &self.selected_thread_id {
                                // ThreadView will handle the case where thread doesn't exist
                                let thread_resp = ThreadView::new(
                                    threads,
                                    thread_id,
                                    note_options,
                                    note_context,
                                    jobs,
                                    col,
                                )
                                .ui(ui);

                                if let Some(action) = thread_resp.output {
                                    note_action = Some(action);
                                }
                            }
                        });
                    });
            });

        (panel_action, note_action)
    }
}

impl Default for ThreadPanel {
    fn default() -> Self {
        Self::new()
    }
}
