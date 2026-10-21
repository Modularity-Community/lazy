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

    iced::application::timed(
        AppState::new,
        AppState::update,
        AppState::subscription,
        AppState::view,
    )
    .theme(Theme::Dark)
    .title("Lazy")
    .run()
}
