use egui::{RichText, TextEdit, Vec2};

use notedeck::{tr, Localization};

pub struct ChannelDialog {
    pub name: String,
    pub hashtags: String,
    pub is_open: bool,
}

pub enum ChannelDialogAction {
    Create { name: String, hashtags: Vec<String> },
    Cancel,
}

impl ChannelDialog {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            hashtags: String::new(),
            is_open: false,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.name.clear();
        self.hashtags.clear();
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        i18n: &mut Localization,
    ) -> Option<ChannelDialogAction> {
        if !self.is_open {
            return None;
        }

        let mut action: Option<ChannelDialogAction> = None;

        egui::Window::new(tr!(i18n, "Create Channel", "Dialog title for creating a new channel"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(400.0, 300.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(16.0);

                    // Channel name input
                    ui.label(
                        RichText::new(tr!(i18n, "Channel Name", "Label for channel name input"))
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    let name_response = ui.add(
                        TextEdit::singleline(&mut self.name)
                            .hint_text(tr!(
                                i18n,
                                "e.g., General, Bitcoin, News...",
                                "Placeholder for channel name"
                            ))
                            .desired_width(f32::INFINITY),
                    );

                    // Auto-focus on name field when opened
                    if name_response.changed() {
                        name_response.request_focus();
                    }

                    ui.add_space(16.0);

                    // Hashtags input
                    ui.label(
                        RichText::new(tr!(i18n, "Hashtags", "Label for hashtags input"))
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(tr!(
                            i18n,
                            "Comma-separated hashtags to track",
                            "Help text for hashtags input"
                        ))
                        .size(12.0)
                        .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(8.0);

                    ui.add(
                        TextEdit::multiline(&mut self.hashtags)
                            .hint_text(tr!(
                                i18n,
                                "e.g., bitcoin, nostr, news",
                                "Placeholder for hashtags"
                            ))
                            .desired_width(f32::INFINITY)
                            .desired_rows(3),
                    );

                    ui.add_space(24.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Create button
                            let create_enabled = !self.name.trim().is_empty()
                                && !self.hashtags.trim().is_empty();

                            let create_button = egui::Button::new(
                                RichText::new(tr!(i18n, "Create", "Button to create channel"))
                                    .size(14.0),
                            )
                            .min_size(Vec2::new(80.0, 32.0));

                            let create_response = ui.add_enabled(create_enabled, create_button);

                            if create_response.clicked() {
                                let hashtags: Vec<String> = self
                                    .hashtags
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();

                                action = Some(ChannelDialogAction::Create {
                                    name: self.name.trim().to_string(),
                                    hashtags,
                                });
                            }

                            ui.add_space(8.0);

                            // Cancel button
                            let cancel_button = egui::Button::new(
                                RichText::new(tr!(i18n, "Cancel", "Button to cancel"))
                                    .size(14.0)
                                    .color(ui.visuals().weak_text_color()),
                            )
                            .frame(false)
                            .min_size(Vec2::new(80.0, 32.0));

                            if ui.add(cancel_button).clicked() {
                                action = Some(ChannelDialogAction::Cancel);
                            }
                        });
                    });
                });
            });

        // Close dialog if action was taken
        if action.is_some() {
            self.close();
        }

        action
    }
}

impl Default for ChannelDialog {
    fn default() -> Self {
        Self::new()
    }
}
