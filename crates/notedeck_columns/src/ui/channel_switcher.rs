use egui::{Color32, Key, Modifiers, RichText, ScrollArea, TextEdit, Vec2};

use notedeck::{tr, Accounts, Localization};

use crate::channels::ChannelsCache;

pub struct ChannelSwitcher {
    pub is_open: bool,
    pub search_query: String,
    pub selected_index: usize,
}

pub enum ChannelSwitcherAction {
    SelectChannel(usize),
    Close,
}

impl ChannelSwitcher {
    pub fn new() -> Self {
        Self {
            is_open: false,
            search_query: String::new(),
            selected_index: 0,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.search_query.clear();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        i18n: &mut Localization,
        channels_cache: &ChannelsCache,
        accounts: &Accounts,
    ) -> Option<ChannelSwitcherAction> {
        if !self.is_open {
            return None;
        }

        let mut action: Option<ChannelSwitcherAction> = None;

        // Modal background
        egui::Area::new(egui::Id::new("channel_switcher_overlay"))
            .fixed_pos(egui::Pos2::ZERO)
            .interactable(true)
            .show(ctx, |ui| {
                let screen_rect = ctx.screen_rect();
                ui.allocate_ui_at_rect(screen_rect, |ui| {
                    // Dark overlay
                    ui.painter().rect_filled(
                        screen_rect,
                        0.0,
                        Color32::from_black_alpha(180),
                    );

                    // Handle click on overlay to close
                    if ui.interact(screen_rect, egui::Id::new("overlay"), egui::Sense::click()).clicked() {
                        action = Some(ChannelSwitcherAction::Close);
                    }
                });
            });

        // Switcher window
        egui::Window::new(tr!(i18n, "Quick Switcher", "Channel switcher dialog title"))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_TOP, Vec2::new(0.0, 100.0))
            .fixed_size(Vec2::new(500.0, 400.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(16.0);

                    // Search input
                    let search_response = ui.add(
                        TextEdit::singleline(&mut self.search_query)
                            .hint_text(tr!(
                                i18n,
                                "Search channels...",
                                "Placeholder for channel search"
                            ))
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Body),
                    );

                    // Auto-focus search field
                    search_response.request_focus();

                    // Handle keyboard navigation
                    ui.input(|i| {
                        if i.key_pressed(Key::Escape) {
                            action = Some(ChannelSwitcherAction::Close);
                        }

                        if i.key_pressed(Key::ArrowDown) {
                            let channels = channels_cache.active_channels(accounts);
                            if self.selected_index < channels.num_channels().saturating_sub(1) {
                                self.selected_index += 1;
                            }
                        }

                        if i.key_pressed(Key::ArrowUp) {
                            self.selected_index = self.selected_index.saturating_sub(1);
                        }

                        if i.key_pressed(Key::Enter) {
                            action = Some(ChannelSwitcherAction::SelectChannel(self.selected_index));
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Channel list
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let channels = channels_cache.active_channels(accounts);
                            let query_lower = self.search_query.to_lowercase();

                            let mut visible_idx = 0;
                            for (idx, channel) in channels.channels.iter().enumerate() {
                                // Filter by search query
                                if !query_lower.is_empty()
                                    && !channel.name.to_lowercase().contains(&query_lower)
                                {
                                    continue;
                                }

                                let is_selected = visible_idx == self.selected_index;
                                let is_current = idx == channels.selected;

                                let mut frame = egui::Frame::new()
                                    .inner_margin(egui::Margin::symmetric(12, 8))
                                    .corner_radius(4.0);

                                if is_selected {
                                    frame = frame.fill(ui.visuals().selection.bg_fill);
                                } else if is_current {
                                    frame = frame.fill(ui.visuals().faint_bg_color);
                                }

                                let response = frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Channel icon
                                        ui.label(RichText::new("# ").size(16.0));

                                        // Channel name
                                        let mut text = RichText::new(&channel.name).size(14.0);
                                        if is_selected {
                                            text = text.strong();
                                        }
                                        ui.label(text);

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Show unread badge if any
                                            if channel.unread_count > 0 {
                                                let count_text = if channel.unread_count > 99 {
                                                    "99+".to_string()
                                                } else {
                                                    channel.unread_count.to_string()
                                                };

                                                ui.label(
                                                    RichText::new(count_text)
                                                        .size(11.0)
                                                        .color(ui.visuals().strong_text_color()),
                                                );
                                            }
                                        });
                                    });
                                });

                                // Handle click on channel
                                let full_response = ui.interact(
                                    response.response.rect,
                                    egui::Id::new(("channel_item", idx)),
                                    egui::Sense::click(),
                                );

                                if full_response.clicked() {
                                    action = Some(ChannelSwitcherAction::SelectChannel(idx));
                                }

                                // Update selected index on hover
                                if full_response.hovered() {
                                    self.selected_index = visible_idx;
                                }

                                visible_idx += 1;
                            }

                            // Show empty state if no results
                            if visible_idx == 0 && !query_lower.is_empty() {
                                ui.add_space(20.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(tr!(
                                            i18n,
                                            "No channels found",
                                            "Empty search results"
                                        ))
                                        .size(14.0)
                                        .color(ui.visuals().weak_text_color()),
                                    );
                                });
                            }
                        });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Help text
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.label(
                            RichText::new(tr!(i18n, "↑↓ to navigate", "Keyboard shortcut hint"))
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.label(
                            RichText::new(tr!(i18n, "↵ to select", "Keyboard shortcut hint"))
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.label(
                            RichText::new(tr!(i18n, "esc to close", "Keyboard shortcut hint"))
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                });
            });

        // Close switcher if action was taken
        if action.is_some() {
            self.close();
        }

        action
    }
}

impl Default for ChannelSwitcher {
    fn default() -> Self {
        Self::new()
    }
}
