use egui::{
    vec2, Color32, CursorIcon, Margin, Rect, RichText, ScrollArea,
    Separator, Stroke, TextStyle, Widget,
};

use crate::channels::ChannelsCache;

use notedeck::{tr, Accounts, Localization};
use notedeck_ui::colors;

pub static CHANNEL_SIDEBAR_WIDTH: f32 = 240.0;

pub struct ChannelSidebar<'a> {
    channels_cache: &'a ChannelsCache,
    accounts: &'a Accounts,
    i18n: &'a mut Localization,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ChannelSidebarAction {
    SelectChannel(usize),
    AddChannel,
    DeleteChannel(usize),
    EditChannel(usize),
}

pub struct ChannelSidebarResponse {
    pub response: egui::Response,
    pub action: ChannelSidebarAction,
}

impl ChannelSidebarResponse {
    fn new(action: ChannelSidebarAction, response: egui::Response) -> Self {
        ChannelSidebarResponse { action, response }
    }
}

impl<'a> ChannelSidebar<'a> {
    pub fn new(
        channels_cache: &'a ChannelsCache,
        accounts: &'a Accounts,
        i18n: &'a mut Localization,
    ) -> Self {
        Self {
            channels_cache,
            accounts,
            i18n,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<ChannelSidebarResponse> {
        let frame =
            egui::Frame::new().inner_margin(Margin::symmetric(12, 16));

        // Background color
        if !ui.visuals().dark_mode {
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect(
                rect,
                0,
                colors::ALMOST_WHITE,
                Stroke::new(0.0, Color32::TRANSPARENT),
                egui::StrokeKind::Inside,
            );
        }

        frame.show(ui, |ui| self.show_inner(ui)).inner
    }

    fn show_inner(&mut self, ui: &mut egui::Ui) -> Option<ChannelSidebarResponse> {
        let channel_list = self.channels_cache.active_channels(self.accounts);
        let selected_index = channel_list.selected;

        ui.vertical(|ui| {
            // Header
            ui.add_space(8.0);
            ui.heading(RichText::new(tr!(
                self.i18n,
                "Channels",
                "Header for channels sidebar"
            ))
            .size(18.0)
            .strong());

            ui.add_space(8.0);
            ui.add(Separator::default().horizontal().spacing(0.0));
            ui.add_space(8.0);

            // Scrollable channel list
            let scroll_response = ScrollArea::vertical()
                .id_salt("channel_list")
                .show(ui, |ui| {
                    let mut selected_action = None;

                    for (index, channel) in channel_list.channels.iter().enumerate() {
                        let is_selected = index == selected_index;
                        let resp = channel_item(ui, &channel.name, is_selected, channel.unread_count, channel_list.num_channels(), index, self.i18n);

                        match resp {
                            ChannelItemResponse::Select => {
                                selected_action = Some(ChannelSidebarAction::SelectChannel(index));
                            }
                            ChannelItemResponse::Delete => {
                                selected_action = Some(ChannelSidebarAction::DeleteChannel(index));
                            }
                            ChannelItemResponse::Edit => {
                                selected_action = Some(ChannelSidebarAction::EditChannel(index));
                            }
                            ChannelItemResponse::None => {}
                        }
                    }

                    selected_action
                })
                .inner;

            if let Some(action) = scroll_response {
                return Some(action);
            }

            ui.add_space(8.0);

            // Add channel button at the bottom
            let add_channel_resp = ui.add(add_channel_button(self.i18n));

            if add_channel_resp.clicked() {
                Some(ChannelSidebarAction::AddChannel)
            } else {
                None
            }
        })
        .inner
        .map(|action| {
            // We need to create a dummy response for ChannelSidebarResponse
            // Use the UI's interact_rect to create a valid response
            let dummy_rect = ui.available_rect_before_wrap();
            let dummy_response = ui.interact(dummy_rect, ui.id().with("channel_sidebar"), egui::Sense::hover());
            ChannelSidebarResponse::new(action, dummy_response)
        })
    }
}

enum ChannelItemResponse {
    Select,
    Delete,
    Edit,
    None,
}

fn channel_item(
    ui: &mut egui::Ui,
    name: &str,
    is_selected: bool,
    unread_count: usize,
    total_channels: usize,
    _channel_index: usize,
    i18n: &mut Localization,
) -> ChannelItemResponse {
    let desired_size = vec2(ui.available_width(), 36.0);

    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let mut action = ChannelItemResponse::None;

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let bg_color = if is_selected {
            if ui.visuals().dark_mode {
                Color32::from_rgb(45, 55, 72) // Darker blue for dark mode
            } else {
                Color32::from_rgb(219, 234, 254) // Light blue for light mode
            }
        } else if response.hovered() {
            if ui.visuals().dark_mode {
                Color32::from_rgb(30, 41, 59)
            } else {
                Color32::from_rgb(243, 244, 246)
            }
        } else {
            Color32::TRANSPARENT
        };

        // Draw background
        if bg_color != Color32::TRANSPARENT {
            ui.painter().rect(
                rect,
                4.0, // rounded corners
                bg_color,
                Stroke::NONE,
                egui::StrokeKind::Inside,
            );
        }

        // Draw hashtag icon
        let icon_rect = egui::Rect::from_min_size(
            rect.min + vec2(8.0, rect.height() / 2.0 - 8.0),
            vec2(16.0, 16.0),
        );
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            "#",
            egui::TextStyle::Body.resolve(ui.style()),
            visuals.text_color(),
        );

        // Draw channel name
        let text_rect = egui::Rect::from_min_size(
            rect.min + vec2(32.0, 0.0),
            vec2(rect.width() - 64.0, rect.height()),
        );
        let text_color = if is_selected {
            if ui.visuals().dark_mode {
                Color32::WHITE
            } else {
                Color32::from_rgb(30, 58, 138)
            }
        } else {
            visuals.text_color()
        };

        ui.painter().text(
            text_rect.left_center(),
            egui::Align2::LEFT_CENTER,
            name,
            egui::TextStyle::Body.resolve(ui.style()),
            text_color,
        );

        // Draw unread badge if there are unread messages
        if unread_count > 0 {
            let badge_text = if unread_count > 99 {
                "99+".to_string()
            } else {
                unread_count.to_string()
            };

            let badge_size = vec2(24.0, 18.0);
            let badge_rect = egui::Rect::from_min_size(
                rect.max - vec2(badge_size.x + 8.0, rect.height() / 2.0 + badge_size.y / 2.0),
                badge_size,
            );

            ui.painter().rect(
                badge_rect,
                9.0,
                Color32::from_rgb(239, 68, 68), // Red badge
                Stroke::NONE,
                egui::StrokeKind::Inside,
            );

            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                &badge_text,
                egui::TextStyle::Small.resolve(ui.style()),
                Color32::WHITE,
            );
        }
    }

    // Handle clicks
    if response.clicked() {
        action = ChannelItemResponse::Select;
    }

    // Show context menu on right-click
    response.context_menu(|ui| {
        if ui.button(tr!(i18n, "Edit Channel", "Context menu option to edit channel")).clicked() {
            action = ChannelItemResponse::Edit;
            ui.close_menu();
        }

        // Only allow delete if not the last channel
        if total_channels > 1 {
            if ui.button(tr!(i18n, "Delete Channel", "Context menu option to delete channel")).clicked() {
                action = ChannelItemResponse::Delete;
                ui.close_menu();
            }
        }
    });

    action
}

fn add_channel_button(i18n: &mut Localization) -> impl Widget + '_ {
    move |ui: &mut egui::Ui| {
        let desired_size = vec2(ui.available_width(), 36.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // Background on hover
            if response.hovered() {
                let bg_color = if ui.visuals().dark_mode {
                    Color32::from_rgb(30, 41, 59)
                } else {
                    Color32::from_rgb(243, 244, 246)
                };
                ui.painter().rect(
                    rect,
                    4.0,
                    bg_color,
                    Stroke::NONE,
                    egui::StrokeKind::Inside,
                );
            }

            // Draw + icon
            let icon_rect = Rect::from_min_size(
                rect.min + vec2(8.0, rect.height() / 2.0 - 8.0),
                vec2(16.0, 16.0),
            );
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                TextStyle::Body.resolve(ui.style()),
                visuals.text_color(),
            );

            // Draw text
            let text_rect = Rect::from_min_size(
                rect.min + vec2(32.0, 0.0),
                vec2(rect.width() - 32.0, rect.height()),
            );
            ui.painter().text(
                text_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                tr!(i18n, "Add Channel", "Button to add a new channel"),
                TextStyle::Body.resolve(ui.style()),
                visuals.text_color(),
            );
        }

        response.on_hover_cursor(CursorIcon::PointingHand)
    }
}
