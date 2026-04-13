mod app;
mod colors;
#[allow(dead_code)]
mod icons;
mod locales;
mod project_views;
mod tasks;
mod theme;
mod types;
mod update;
mod views;

use app::LexitoApp;
use iced::{Color, Font};

/// Top padding for content when macOS titlebar is hidden (traffic lights area).
#[cfg(target_os = "macos")]
pub const TITLEBAR_TOP_PAD: f32 = 36.0;
#[cfg(not(target_os = "macos"))]
pub const TITLEBAR_TOP_PAD: f32 = 0.0;

/// Left padding to clear macOS traffic lights.
#[cfg(target_os = "macos")]
pub const TITLEBAR_LEFT_PAD: f32 = 16.0;
#[cfg(not(target_os = "macos"))]
pub const TITLEBAR_LEFT_PAD: f32 = 0.0;

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    let platform = iced::window::settings::PlatformSpecific {
        title_hidden: true,
        titlebar_transparent: true,
        fullsize_content_view: true,
    };
    #[cfg(not(target_os = "macos"))]
    let platform = iced::window::settings::PlatformSpecific::default();

    iced::application(LexitoApp::boot, LexitoApp::update, LexitoApp::view)
        .title(LexitoApp::title)
        .theme(LexitoApp::theme)
        .font(include_bytes!("fonts/Inter-Regular.ttf").as_slice())
        .font(include_bytes!("fonts/Inter-Bold.ttf").as_slice())
        .font(include_bytes!("fonts/JetBrainsMono-Regular.ttf").as_slice())
        .default_font(Font::with_name("Inter"))
        .subscription(LexitoApp::subscription)
        .transparent(true)
        .window(iced::window::Settings {
            size: iced::Size::new(1440.0, 920.0),
            position: iced::window::Position::Centered,
            transparent: true,
            blur: true,
            platform_specific: platform,
            ..Default::default()
        })
        .style(|_state, theme| {
            let palette = theme.extended_palette();
            let alpha = if palette.is_dark { 0.92 } else { 1.0 };
            iced::theme::Style {
                background_color: Color {
                    a: alpha,
                    ..palette.background.base.color
                },
                text_color: palette.background.base.text,
            }
        })
        .run()
}
