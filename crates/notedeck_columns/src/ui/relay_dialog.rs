use egui::{RichText, TextEdit, Vec2};

use notedeck::{tr, Localization};

pub struct RelayDialog {
    pub relay_url: String,
    pub is_open: bool,
    pub focus_requested: bool,
}

pub enum RelayDialogAction {
    Add { url: String },
    Cancel,
}

impl RelayDialog {
    pub fn new() -> Self {
        Self {
            relay_url: String::new(),
            is_open: false,
            focus_requested: false,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.relay_url.clear();
        self.focus_requested = false;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        i18n: &mut Localization,
    ) -> Option<RelayDialogAction> {
        if !self.is_open {
            return None;
        }

        let mut action: Option<RelayDialogAction> = None;

        let title = tr!(i18n, "Add Relay", "Dialog title for adding a relay");

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(450.0, 200.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(16.0);

                    // Relay URL input
                    ui.label(
                        RichText::new(tr!(i18n, "Relay URL", "Label for relay URL input"))
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(tr!(
                            i18n,
                            "Enter a relay URL to connect to the nostr network",
                            "Help text for relay URL input"
                        ))
                        .size(12.0)
                        .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(8.0);

                    let url_response = ui.add(
                        TextEdit::singleline(&mut self.relay_url)
                            .hint_text(tr!(
                                i18n,
                                "wss://relay.example.com",
                                "Placeholder for relay URL"
                            ))
                            .desired_width(f32::INFINITY),
                    );

                    // Auto-focus on URL field when first opened
                    if !self.focus_requested {
                        url_response.request_focus();
                        self.focus_requested = true;
                    }

                    // Handle Escape and Enter keys
                    let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if escape_pressed {
                        action = Some(RelayDialogAction::Cancel);
                    }

                    if enter_pressed && !self.relay_url.trim().is_empty() {
                        action = Some(RelayDialogAction::Add {
                            url: self.relay_url.trim().to_string(),
                        });
                    }

                    ui.add_space(24.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Add button
                            let button_enabled = !self.relay_url.trim().is_empty();

                            let button = egui::Button::new(
                                RichText::new(tr!(i18n, "Add", "Button to add relay"))
                                    .size(14.0),
                            )
                            .min_size(Vec2::new(80.0, 32.0));

                            let button_response = ui.add_enabled(button_enabled, button);

                            if button_response.clicked() {
                                action = Some(RelayDialogAction::Add {
                                    url: self.relay_url.trim().to_string(),
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
                                action = Some(RelayDialogAction::Cancel);
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

impl Default for RelayDialog {
    fn default() -> Self {
        Self::new()
    }
}
