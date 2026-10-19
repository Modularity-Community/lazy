use iced::Theme;

use crate::ui::app::AppState;

mod db;
mod domain;
mod enums;
mod infra;
mod logger;
mod ui;
mod util;

fn main() -> iced::Result {
    let _guard = logger::init_logger();

    tracing::info!("Starting App Lazy..");

    iced::application("Lazy", AppState::update, AppState::view)
        .theme(|_| Theme::Dark)
        .run_with(AppState::new)
}
