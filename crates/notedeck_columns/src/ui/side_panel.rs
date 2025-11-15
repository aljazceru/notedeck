use egui::{vec2, CursorIcon, InnerResponse, Layout, Margin, Separator, Stroke, Widget};
use tracing::info;

use crate::{
    accounts::AccountsRoute, app::get_active_columns_mut, decks::DecksCache,
    nav::SwitchingAction, route::Route,
};

use notedeck::{Accounts, Localization, UserAccount};
use notedeck_ui::{
    anim::{AnimationHelper, ICON_EXPANSION_MULTIPLE},
    app_images, colors, View,
};

pub static SIDE_PANEL_WIDTH: f32 = 68.0;
static ICON_WIDTH: f32 = 40.0;

pub struct DesktopSidePanel<'a> {
    selected_account: &'a UserAccount,
}

impl View for DesktopSidePanel<'_> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.show(ui);
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SidePanelAction {
    Columns,
    ComposeNote,
    Search,
    ExpandSidePanel,
    Wallet,
    Account,  // Use existing Account instead of UserAccount
    Settings,
}

pub struct SidePanelResponse {
    pub response: egui::Response,
    pub action: SidePanelAction,
}

impl SidePanelResponse {
    fn new(action: SidePanelAction, response: egui::Response) -> Self {
        SidePanelResponse { action, response }
    }
}

impl<'a> DesktopSidePanel<'a> {
    pub fn new(selected_account: &'a UserAccount) -> Self {
        Self { selected_account }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<SidePanelResponse> {
        let frame =
            egui::Frame::new().inner_margin(Margin::same(notedeck_ui::constants::FRAME_MARGIN));

        if !ui.visuals().dark_mode {
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect(
                rect,
                0,
                colors::ALMOST_WHITE,
                egui::Stroke::new(0.0, egui::Color32::TRANSPARENT),
                egui::StrokeKind::Inside,
            );
        }

        frame.show(ui, |ui| self.show_inner(ui)).inner
    }

    fn show_inner(&mut self, ui: &mut egui::Ui) -> Option<SidePanelResponse> {
        let dark_mode = ui.ctx().style().visuals.dark_mode;

        let inner = ui
            .vertical(|ui| {
                ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                    // User controls section
                    ui.add_space(4.0);

                    // User profile button
                    let user_account_resp = ui.add(user_account_button(&self.selected_account, dark_mode));

                    // Add account button if no account or for switching
                    let add_account_resp = ui.add(add_account_button(dark_mode));

                    // Relay configuration button
                    let relay_resp = ui.add(relay_button(dark_mode));

                    ui.add(Separator::default().horizontal().spacing(8.0).shrink(4.0));

                    // macos needs a bit of space to make room for window
                    // minimize/close buttons
                    //if cfg!(target_os = "macos") {
                    //    ui.add_space(24.0);
                    //}

                    let compose_resp = ui
                        .add(crate::ui::post::compose_note_button(dark_mode))
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    let search_resp = ui.add(search_button());
                    let column_resp = ui.add(add_column_button());

                    /*
                    if expand_resp.clicked() {
                        Some(InnerResponse::new(
                            SidePanelAction::ExpandSidePanel,
                            expand_resp,
                        ))
                    */
                    if user_account_resp.clicked() {
                        Some(InnerResponse::new(
                            SidePanelAction::Account,
                            user_account_resp,
                        ))
                    } else if add_account_resp.clicked() {
                        Some(InnerResponse::new(
                            SidePanelAction::Account, // Reuse Account for adding account
                            add_account_resp,
                        ))
                    } else if relay_resp.clicked() {
                        Some(InnerResponse::new(SidePanelAction::Settings, relay_resp))
                    } else if compose_resp.clicked() {
                        Some(InnerResponse::new(
                            SidePanelAction::ComposeNote,
                            compose_resp,
                        ))
                    } else if search_resp.clicked() {
                        Some(InnerResponse::new(SidePanelAction::Search, search_resp))
                    } else if column_resp.clicked() {
                        Some(InnerResponse::new(SidePanelAction::Columns, column_resp))
                    } else {
                        None
                    }
                })
                .inner
            })
            .inner;

        if let Some(inner) = inner {
            Some(SidePanelResponse::new(inner.inner, inner.response))
        } else {
            None
        }
    }

    pub fn perform_action(
        decks_cache: &mut DecksCache,
        accounts: &Accounts,
        action: SidePanelAction,
        i18n: &mut Localization,
    ) -> Option<SwitchingAction> {
        let router = get_active_columns_mut(i18n, accounts, decks_cache).get_selected_router();
        let switching_response = None;
        match action {
            SidePanelAction::Account => {
                if router
                    .routes()
                    .iter()
                    .any(|r| r == &Route::Accounts(AccountsRoute::Accounts))
                {
                    // return if we are already routing to accounts
                    router.go_back();
                } else {
                    router.route_to(Route::accounts());
                }
            }
            SidePanelAction::Settings => {
                if router.routes().iter().any(|r| r == &Route::Relays) {
                    // return if we are already routing to relays
                    router.go_back();
                } else {
                    router.route_to(Route::relays());
                }
            }
            SidePanelAction::Columns => {
                if router
                    .routes()
                    .iter()
                    .any(|r| matches!(r, Route::AddColumn(_)))
                {
                    router.go_back();
                } else {
                    get_active_columns_mut(i18n, accounts, decks_cache).new_column_picker();
                }
            }
            SidePanelAction::ComposeNote => {
                let can_post = accounts.get_selected_account().key.secret_key.is_some();

                if !can_post {
                    router.route_to(Route::accounts());
                } else if router.routes().iter().any(|r| r == &Route::ComposeNote) {
                    router.go_back();
                } else {
                    router.route_to(Route::ComposeNote);
                }
            }
            SidePanelAction::Search => {
                // TODO
                if router.top() == &Route::Search {
                    router.go_back();
                } else {
                    router.route_to(Route::Search);
                }
            }
            SidePanelAction::ExpandSidePanel => {
                // TODO
                info!("Clicked expand side panel button");
            }
            SidePanelAction::Wallet => 's: {
                if router
                    .routes()
                    .iter()
                    .any(|r| matches!(r, Route::Wallet(_)))
                {
                    router.go_back();
                    break 's;
                }

                router.route_to(Route::Wallet(notedeck::WalletType::Auto));
            }
        }
        switching_response
    }
}

fn add_column_button() -> impl Widget {
    move |ui: &mut egui::Ui| {
        let img_size = 24.0;
        let max_size = ICON_WIDTH * ICON_EXPANSION_MULTIPLE; // max size of the widget

        let img = if ui.visuals().dark_mode {
            app_images::add_column_dark_image()
        } else {
            app_images::add_column_light_image()
        };

        let helper = AnimationHelper::new(ui, "add-column-button", vec2(max_size, max_size));

        let cur_img_size = helper.scale_1d_pos(img_size);
        img.paint_at(
            ui,
            helper
                .get_animation_rect()
                .shrink((max_size - cur_img_size) / 2.0),
        );

        helper
            .take_animation_response()
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text("Add new column")
    }
}

pub fn search_button_impl(color: egui::Color32, line_width: f32) -> impl Widget {
    move |ui: &mut egui::Ui| -> egui::Response {
        let max_size = ICON_WIDTH * ICON_EXPANSION_MULTIPLE; // max size of the widget
        let min_line_width_circle = line_width; // width of the magnifying glass
        let min_line_width_handle = line_width;
        let helper = AnimationHelper::new(ui, "search-button", vec2(max_size, max_size));

        let painter = ui.painter_at(helper.get_animation_rect());

        let cur_line_width_circle = helper.scale_1d_pos(min_line_width_circle);
        let cur_line_width_handle = helper.scale_1d_pos(min_line_width_handle);
        let min_outer_circle_radius = helper.scale_radius(15.0);
        let cur_outer_circle_radius = helper.scale_1d_pos(min_outer_circle_radius);
        let min_handle_length = 7.0;
        let cur_handle_length = helper.scale_1d_pos(min_handle_length);

        let circle_center = helper.scale_from_center(-2.0, -2.0);

        let handle_vec = vec2(
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
        );

        let handle_pos_1 = circle_center + (handle_vec * (cur_outer_circle_radius - 3.0));
        let handle_pos_2 =
            circle_center + (handle_vec * (cur_outer_circle_radius + cur_handle_length));

        let circle_stroke = Stroke::new(cur_line_width_circle, color);
        let handle_stroke = Stroke::new(cur_line_width_handle, color);

        painter.line_segment([handle_pos_1, handle_pos_2], handle_stroke);
        painter.circle(
            circle_center,
            min_outer_circle_radius,
            ui.style().visuals.widgets.inactive.weak_bg_fill,
            circle_stroke,
        );

        helper
            .take_animation_response()
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text("Open search")
    }
}

pub fn search_button() -> impl Widget {
    search_button_impl(colors::MID_GRAY, 1.5)
}

fn user_account_button(user_account: &UserAccount, dark_mode: bool) -> impl Widget + '_ {
    move |ui: &mut egui::Ui| -> egui::Response {
        let max_size = ICON_WIDTH * ICON_EXPANSION_MULTIPLE;
        let helper = AnimationHelper::new(ui, "user-account-button", vec2(max_size, max_size));

        // Show user icon if logged in, otherwise show login icon
        let img = if user_account.key.secret_key.is_some() {
            // User is logged in - show user/home icon
            if dark_mode {
                app_images::home_dark_image()
            } else {
                app_images::home_light_image()
            }
        } else {
            // No user logged in - show login icon
            if dark_mode {
                app_images::link_dark_image()
            } else {
                app_images::link_light_image()
            }
        };

        let cur_img_size = helper.scale_1d_pos(ICON_WIDTH - 8.0); // Slightly smaller
        img.paint_at(
            ui,
            helper
                .get_animation_rect()
                .shrink((max_size - cur_img_size) / 2.0),
        );

        helper
            .take_animation_response()
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(if user_account.key.secret_key.is_some() {
                "User Account"
            } else {
                "Login / Create Account"
            })
    }
}

fn add_account_button(dark_mode: bool) -> impl Widget {
    move |ui: &mut egui::Ui| -> egui::Response {
        let max_size = ICON_WIDTH * ICON_EXPANSION_MULTIPLE;
        let helper = AnimationHelper::new(ui, "add-account-button", vec2(max_size, max_size));

        let img = if dark_mode {
            app_images::add_column_dark_image()
        } else {
            app_images::add_column_light_image()
        };

        let cur_img_size = helper.scale_1d_pos(ICON_WIDTH - 12.0);
        img.paint_at(
            ui,
            helper
                .get_animation_rect()
                .shrink((max_size - cur_img_size) / 2.0),
        );

        helper
            .take_animation_response()
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text("Add Account")
    }
}

fn relay_button(dark_mode: bool) -> impl Widget {
    move |ui: &mut egui::Ui| -> egui::Response {
        let max_size = ICON_WIDTH * ICON_EXPANSION_MULTIPLE;
        let helper = AnimationHelper::new(ui, "relay-button", vec2(max_size, max_size));

        let img = if dark_mode {
            app_images::reply_dark_image()
        } else {
            app_images::reply_light_image()
        };

        let cur_img_size = helper.scale_1d_pos(ICON_WIDTH - 10.0);
        img.paint_at(
            ui,
            helper
                .get_animation_rect()
                .shrink((max_size - cur_img_size) / 2.0),
        );

        helper
            .take_animation_response()
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text("Relay Configuration")
    }
}
