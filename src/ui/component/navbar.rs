use iced::{
    Element, color, widget::Space, widget::button, widget::container, widget::horizontal_space,
    widget::row, widget::text,
};

pub struct NavbarState {}

#[derive(Debug, Clone)]
pub enum NavbarMessage {}

impl NavbarState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, _: NavbarMessage) {}

    pub fn view(&mut self) -> Element<NavbarMessage> {
        let logo = text("Lazy").size(24);
        let tabs = row![
            button(text("Collections")).padding([5, 10]),
            button(text("History")).padding([5, 10]),
            button(text("Environments")).padding([5, 10]),
        ]
        .spacing(15);

        let settings_icon = text("icon_setting");

        container(
            row![
                logo,
                Space::with_width(30),
                tabs,
                horizontal_space(),
                settings_icon
            ]
            .align_y(iced::Alignment::Center)
            .padding([10, 20]),
        )
        .style(|_theme| container::background(color!(0x1A1B26)))
        .into()
    }
}
