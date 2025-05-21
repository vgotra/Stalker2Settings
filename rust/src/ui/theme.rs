// Theme module for Stalker2Settings
// Contains theme-related functionality

use cursive::theme::{BorderStyle, Theme, Color, PaletteColor};

/// Create a custom theme for the application
pub fn create_theme() -> Theme {
    let mut theme = Theme::default();
    theme.shadow = false;
    theme.borders = BorderStyle::Simple;
    theme.palette[PaletteColor::Background] = Color::TerminalDefault;
    theme.palette[PaletteColor::View] = Color::TerminalDefault;
    theme.palette[PaletteColor::Primary] = Color::Dark(cursive::theme::BaseColor::Blue);
    theme.palette[PaletteColor::Secondary] = Color::Light(cursive::theme::BaseColor::Blue);
    theme.palette[PaletteColor::TitlePrimary] = Color::Light(cursive::theme::BaseColor::Blue);
    theme.palette[PaletteColor::Highlight] = Color::Dark(cursive::theme::BaseColor::Blue);
    theme
}