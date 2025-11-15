use egui::{FontFamily, FontId};

use notedeck::fonts::NamedFontFamily;

pub fn deck_icon_font_sized(size: f32) -> FontId {
    egui::FontId::new(size, emoji_font_family())
}

pub fn emoji_font_family() -> FontFamily {
    egui::FontFamily::Name(NamedFontFamily::Emoji.as_str().into())
}
