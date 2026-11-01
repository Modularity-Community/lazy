use iced::{Settings, Theme};

use crate::ui::app::AppState;

mod domain;
mod enums;
mod infra;
mod logger;
mod ui;

fn main() -> iced::Result {
    let _guard = logger::init_logger();

    tracing::info!("Starting App Lazy..");

    let font_bytes = include_bytes!("../resources/FiraCode-Regular.ttf");

    let global_settings = Settings {
        fonts: vec![font_bytes.into()],
        ..Default::default()
    };

    iced::application::timed(
        AppState::new,
        AppState::update,
        AppState::subscription,
        AppState::view,
    )
    .settings(global_settings)
    .default_font(iced::Font {
        family: iced::font::Family::Name("Fira Code"),
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    })
    .theme(Theme::Dark)
    .title("Lazy")
    .run()
}
