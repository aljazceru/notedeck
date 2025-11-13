use egui::{
    vec2, Align, Color32, CursorIcon, Layout, Margin, RichText, ScrollArea, Sense,
    Stroke,
};
use nostrdb::{Filter, Note, NoteKey, Transaction};
use notedeck::fonts::get_font_size;
use notedeck::name::get_display_name;
use notedeck::note::ReactAction;
use notedeck::{tr, JobsCache, NoteAction, NoteContext, NotedeckTextStyle};
use notedeck_ui::{app_images, ProfilePic};
use tracing::warn;

use crate::nav::BodyResponse;
use crate::timeline::{TimelineCache, TimelineKind};
use notedeck_ui::NoteOptions;

const MESSAGE_BUBBLE_PADDING: i8 = 12;
const MESSAGE_SPACING: f32 = 8.0;
const GROUP_SPACING: f32 = 16.0;
const AVATAR_SIZE: f32 = 36.0;
const MAX_BUBBLE_WIDTH_RATIO: f32 = 0.75; // 75% of available width

struct MessageBubbleResponse {
    action: Option<NoteAction>,
    hovered: bool,
}

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

                        let action = self.render_message(ui, &note, &txn, note_key, !same_group);
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
        note_key: NoteKey,
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
                let bubble_response = self.render_message_bubble(ui, note, txn);
                if bubble_response.action.is_some() && note_action.is_none() {
                    note_action = bubble_response.action;
                }

                // Interaction bar (show on hover)
                if bubble_response.hovered {
                    ui.add_space(4.0);
                    let action_bar_resp = self.render_action_bar(ui, note, txn, note_key);
                    if action_bar_resp.is_some() && note_action.is_none() {
                        note_action = action_bar_resp;
                    }
                }
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
            let timestamp = format_timestamp(note.created_at(), self.note_context.i18n);
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
    ) -> MessageBubbleResponse {
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

        let hovered = response.hovered();

        // Hover effect
        if hovered {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        MessageBubbleResponse {
            action: note_action,
            hovered,
        }
    }

    fn render_action_bar(&mut self, ui: &mut egui::Ui, note: &Note, txn: &Transaction, note_key: NoteKey) -> Option<NoteAction> {
        let mut action: Option<NoteAction> = None;
        let spacing = 16.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.set_min_height(24.0);

            // Reply button
            let reply_resp =
                self.reply_button(ui, note_key).on_hover_cursor(egui::CursorIcon::PointingHand);

            if reply_resp.clicked() {
                action = Some(NoteAction::Reply(enostr::NoteId::new(*note.id())));
            }

            ui.add_space(spacing);

            // Like button
            let current_user_pubkey = self.note_context.accounts.selected_account_pubkey();
            // Query nostrdb to check if user has already reacted to this note
            let filled = has_user_reacted(self.note_context.ndb, txn, &current_user_pubkey, note.id());

            let like_resp =
                self.like_button(ui, note_key, filled).on_hover_cursor(egui::CursorIcon::PointingHand);

            if like_resp.clicked() {
                action = Some(NoteAction::React(ReactAction {
                    note_id: enostr::NoteId::new(*note.id()),
                    content: "+",
                }));
            }

            ui.add_space(spacing);

            // Repost button
            let repost_resp =
                self.repost_button(ui, note_key).on_hover_cursor(egui::CursorIcon::PointingHand);

            if repost_resp.clicked() {
                action = Some(NoteAction::Repost(enostr::NoteId::new(*note.id())));
            }
        });

        action
    }

    fn reply_button(&mut self, ui: &mut egui::Ui, _note_key: NoteKey) -> egui::Response {
        let img = if ui.style().visuals.dark_mode {
            app_images::reply_dark_image()
        } else {
            app_images::reply_light_image()
        };

        ui.add(img.max_width(18.0).sense(Sense::click()))
            .on_hover_text(tr!(
                self.note_context.i18n,
                "Reply to this note",
                "Hover text for reply button"
            ))
    }

    fn like_button(
        &mut self,
        ui: &mut egui::Ui,
        _note_key: NoteKey,
        filled: bool,
    ) -> egui::Response {
        let img = {
            let img = if filled {
                app_images::like_image_filled()
            } else {
                app_images::like_image()
            };

            if ui.visuals().dark_mode {
                img.tint(ui.visuals().text_color())
            } else {
                img
            }
        };

        ui.add(img.max_width(18.0).sense(Sense::click()))
            .on_hover_text(tr!(
                self.note_context.i18n,
                "Like this note",
                "Hover text for like button"
            ))
    }

    fn repost_button(&mut self, ui: &mut egui::Ui, _note_key: NoteKey) -> egui::Response {
        let img = if ui.style().visuals.dark_mode {
            app_images::repost_dark_image()
        } else {
            app_images::repost_light_image()
        };

        ui.add(img.max_width(18.0).sense(Sense::click()))
            .on_hover_text(tr!(
                self.note_context.i18n,
                "Repost this note",
                "Hover text for repost button"
            ))
    }

    fn get_bubble_color(&self, ui: &egui::Ui) -> Color32 {
        if ui.visuals().dark_mode {
            Color32::from_rgb(31, 41, 55) // Dark gray for dark mode
        } else {
            Color32::from_rgb(249, 250, 251) // Light gray for light mode
        }
    }
}

/// Check if the current user has already reacted to a note
fn has_user_reacted(
    ndb: &nostrdb::Ndb,
    txn: &Transaction,
    user_pubkey: &enostr::Pubkey,
    note_id: &[u8; 32],
) -> bool {
    // Query for Kind 7 (reaction) events from this user for this note
    let note_id_hex = enostr::NoteId::new(*note_id).hex();
    let filter = Filter::new()
        .kinds([7])
        .authors([user_pubkey.bytes()])
        .tags([note_id_hex.as_str()], 'e')
        .limit(1)
        .build();

    // Check if any reactions exist
    ndb.query(txn, &[filter], 1)
        .ok()
        .map(|results| !results.is_empty())
        .unwrap_or(false)
}

/// Format timestamp as relative time (e.g., "5m ago", "2h ago", "Yesterday")
fn format_timestamp(created_at: u64, i18n: &mut notedeck::Localization) -> String {
    let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => {
            // System time is before Unix epoch - extremely rare but handle gracefully
            return tr!(i18n, "Unknown time", "Fallback when system time is invalid").to_string();
        }
    };

    let diff = now.saturating_sub(created_at);

    if diff < 60 {
        tr!(i18n, "Just now", "Time less than 1 minute ago").to_string()
    } else if diff < 3600 {
        let minutes = diff / 60;
        format!("{minutes}m {}", tr!(i18n, "ago", "Time suffix"))
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{hours}h {}", tr!(i18n, "ago", "Time suffix"))
    } else if diff < 172800 {
        tr!(i18n, "Yesterday", "One day ago").to_string()
    } else if diff < 604800 {
        let days = diff / 86400;
        format!("{days}d {}", tr!(i18n, "ago", "Time suffix"))
    } else {
        // Simple date format without chrono dependency
        let days = diff / 86400;
        format!("{} {} {}", days, tr!(i18n, "days", "Plural days"), tr!(i18n, "ago", "Time suffix"))
    }
}
