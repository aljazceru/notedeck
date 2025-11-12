use egui::{
    vec2, Align, Color32, CursorIcon, Layout, Margin, RichText, ScrollArea, Sense,
    Stroke,
};
use nostrdb::{Note, Transaction};
use notedeck::fonts::get_font_size;
use notedeck::name::get_display_name;
use notedeck::{tr, JobsCache, NoteAction, NoteContext, NotedeckTextStyle};
use notedeck_ui::ProfilePic;
use tracing::warn;

use crate::nav::BodyResponse;
use crate::timeline::{TimelineCache, TimelineKind};
use notedeck_ui::NoteOptions;

const MESSAGE_BUBBLE_PADDING: i8 = 12;
const MESSAGE_SPACING: f32 = 8.0;
const GROUP_SPACING: f32 = 16.0;
const AVATAR_SIZE: f32 = 36.0;
const MAX_BUBBLE_WIDTH_RATIO: f32 = 0.75; // 75% of available width

pub struct ChatView<'a, 'd> {
    timeline_id: &'a TimelineKind,
    timeline_cache: &'a mut TimelineCache,
    note_options: NoteOptions,
    note_context: &'a mut NoteContext<'d>,
    jobs: &'a mut JobsCache,
    col: usize,
}

impl<'a, 'd> ChatView<'a, 'd> {
    pub fn new(
        timeline_id: &'a TimelineKind,
        timeline_cache: &'a mut TimelineCache,
        note_context: &'a mut NoteContext<'d>,
        note_options: NoteOptions,
        jobs: &'a mut JobsCache,
        col: usize,
    ) -> Self {
        Self {
            timeline_id,
            timeline_cache,
            note_options,
            note_context,
            jobs,
            col,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> BodyResponse<Option<NoteAction>> {
        // Check that timeline exists
        if self.timeline_cache.get(self.timeline_id).is_none() {
            return BodyResponse::none();
        }

        let scroll_id = egui::Id::new(("chat_scroll", self.timeline_id, self.col));

        let mut note_action: Option<NoteAction> = None;

        // Main scroll area for messages
        let _scroll_response = ScrollArea::vertical()
            .id_salt(scroll_id)
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    let units_len = {
                        let timeline = if let Some(tl) = self.timeline_cache.get(self.timeline_id) {
                            tl
                        } else {
                            warn!("Timeline missing in chat view");
                            return;
                        };
                        timeline.current_view().units.len()
                    };

                    let txn = if let Ok(txn) = Transaction::new(self.note_context.ndb) {
                        txn
                    } else {
                        warn!("Failed to create transaction for chat view");
                        return;
                    };

                    if units_len == 0 {
                        // Empty state
                        ui.add_space(50.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(tr!(
                                    self.note_context.i18n,
                                    "No messages yet",
                                    "Empty state message for chat"
                                ))
                                .size(16.0)
                                .color(ui.visuals().weak_text_color()),
                            );
                            ui.label(
                                RichText::new(tr!(
                                    self.note_context.i18n,
                                    "Messages will appear here when they arrive",
                                    "Empty state subtext for chat"
                                ))
                                .size(14.0)
                                .color(ui.visuals().weak_text_color()),
                            );
                        });
                        return;
                    }

                    let mut last_author: Option<Vec<u8>> = None;
                    let mut last_timestamp: u64 = 0;

                    for i in 0..units_len {
                        let note_key = {
                            let timeline = if let Some(tl) = self.timeline_cache.get(self.timeline_id) {
                                tl
                            } else {
                                continue;
                            };

                            let unit = if let Some(u) = timeline.current_view().units.get(i) {
                                u
                            } else {
                                continue;
                            };

                            // Extract the note key from the unit
                            match unit {
                                crate::timeline::NoteUnit::Single(note_ref) => note_ref.key,
                                crate::timeline::NoteUnit::Composite(composite) => {
                                    match composite {
                                        crate::timeline::CompositeUnit::Reaction(r) => r.note_reacted_to.key,
                                        crate::timeline::CompositeUnit::Repost(r) => r.note_reposted.key,
                                    }
                                }
                            }
                        };

                        let note = if let Ok(note) = self.note_context.ndb.get_note_by_key(&txn, note_key) {
                            note
                        } else {
                            continue;
                        };

                        // Check if this is from the same author within a short time window
                        let same_group = if let Some(ref last_auth) = last_author {
                            let author_bytes = note.pubkey();
                            let time_diff = note.created_at().abs_diff(last_timestamp);
                            author_bytes == last_auth.as_slice() && time_diff < 300 // 5 minutes
                        } else {
                            false
                        };

                        if !same_group {
                            ui.add_space(GROUP_SPACING);
                        }

                        let action = self.render_message(ui, &note, &txn, !same_group);
                        if action.is_some() && note_action.is_none() {
                            note_action = action;
                        }

                        last_author = Some(note.pubkey().to_vec());
                        last_timestamp = note.created_at();

                        if !same_group {
                            ui.add_space(MESSAGE_SPACING);
                        } else {
                            ui.add_space(MESSAGE_SPACING / 2.0);
                        }
                    }

                    ui.add_space(16.0); // Bottom padding
                });
            });

        BodyResponse::output(Some(note_action))
    }

    fn render_message(
        &mut self,
        ui: &mut egui::Ui,
        note: &Note,
        txn: &Transaction,
        show_header: bool,
    ) -> Option<NoteAction> {
        let mut note_action: Option<NoteAction> = None;
        let available_width = ui.available_width();
        let max_bubble_width = available_width * MAX_BUBBLE_WIDTH_RATIO;

        ui.horizontal(|ui| {
            // Avatar column (fixed width)
            ui.allocate_ui_with_layout(
                vec2(AVATAR_SIZE + MESSAGE_BUBBLE_PADDING as f32, ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| {
                    if show_header {
                        // Show avatar
                        let profile = self.note_context.ndb
                            .get_profile_by_pubkey(txn, note.pubkey())
                            .ok();

                        let resp = ui.add(
                            &mut ProfilePic::from_profile_or_default(
                                self.note_context.img_cache,
                                profile.as_ref()
                            )
                            .size(AVATAR_SIZE)
                        );

                        if resp.clicked() {
                            note_action = Some(NoteAction::Profile(enostr::Pubkey::new(*note.pubkey())));
                        }
                    } else {
                        // Just spacing for grouped messages
                        ui.add_space(AVATAR_SIZE);
                    }
                },
            );

            // Message content column
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                // Constrain the content width for message bubbles
                ui.set_max_width(max_bubble_width);

                if show_header {
                    // Message header: name + timestamp
                    self.render_message_header(ui, note, txn);
                }

                // Message bubble
                let bubble_action = self.render_message_bubble(ui, note, txn);
                if bubble_action.is_some() && note_action.is_none() {
                    note_action = bubble_action;
                }

                // Interaction bar (hover only)
                // TODO: Add like/reply/repost buttons on hover
            });
        });

        note_action
    }

    fn render_message_header(&mut self, ui: &mut egui::Ui, note: &Note, txn: &Transaction) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Author name
            let profile = self.note_context.ndb
                .get_profile_by_pubkey(txn, note.pubkey())
                .ok();

            let display_name = get_display_name(profile.as_ref());

            let name_response = ui.add(
                egui::Label::new(
                    RichText::new(display_name.name())
                        .size(15.0)
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                )
                .sense(Sense::click()),
            );

            if name_response.clicked() {
                // TODO: Show profile preview
            }

            // Timestamp
            let timestamp = format_timestamp(note.created_at());
            ui.label(
                RichText::new(timestamp)
                    .size(12.0)
                    .color(ui.visuals().weak_text_color()),
            );
        });

        ui.add_space(4.0);
    }

    fn render_message_bubble(
        &mut self,
        ui: &mut egui::Ui,
        note: &Note,
        _txn: &Transaction,
    ) -> Option<NoteAction> {
        let mut note_action: Option<NoteAction> = None;

        let frame = egui::Frame::new()
            .inner_margin(Margin::same(MESSAGE_BUBBLE_PADDING))
            .fill(self.get_bubble_color(ui))
            .corner_radius(8.0)
            .stroke(Stroke::new(
                1.0,
                if ui.visuals().dark_mode {
                    Color32::from_rgb(55, 65, 81)
                } else {
                    Color32::from_rgb(229, 231, 235)
                },
            ));

        let response = frame
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width());

                // Message content
                let content = note.content();

                let text = RichText::new(content)
                    .size(get_font_size(ui.ctx(), &NotedeckTextStyle::Body));

                ui.add(egui::Label::new(text).wrap());
            })
            .response;

        // Make bubble clickable to open thread
        if response.clicked() {
            use enostr::NoteId;
            note_action = Some(NoteAction::note(NoteId::new(*note.id())));
        }

        // Hover effect
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        note_action
    }

    fn get_bubble_color(&self, ui: &egui::Ui) -> Color32 {
        if ui.visuals().dark_mode {
            Color32::from_rgb(31, 41, 55) // Dark gray for dark mode
        } else {
            Color32::from_rgb(249, 250, 251) // Light gray for light mode
        }
    }
}

/// Format timestamp as relative time (e.g., "5m ago", "2h ago", "Yesterday")
fn format_timestamp(created_at: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let diff = now.saturating_sub(created_at);

    if diff < 60 {
        "Just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 172800 {
        "Yesterday".to_string()
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        // Simple date format without chrono dependency
        format!("{} days ago", diff / 86400)
    }
}
