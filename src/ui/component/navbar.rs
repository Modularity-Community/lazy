use iced::{
    Alignment, Element, Length, Task,
    widget::{button, container, responsive, row, space::horizontal, text},
};

use crate::ui::theme::colors::{MUTED, SURFACE, TEXT, VIOLET};
pub struct NavbarState {}

#[derive(Debug, Clone)]
pub enum NavbarMessage {}

impl NavbarState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, _: NavbarMessage) -> Task<NavbarMessage> {
        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, NavbarMessage> {
        let logo = row![
            text("Lazy").size(21).color(TEXT),
            text("BETA").size(9).color(VIOLET),
        ]
        .spacing(7)
        .align_y(Alignment::Center);

        let nav_items: Element<'a, NavbarMessage> = responsive(move |size| {
            let collections = button(text("Collections").size(12).color(VIOLET)).padding([8, 10]);
            let history = button(text("History").size(12).color(MUTED)).padding([8, 10]);
            let environments = button(text("Environments").size(12).color(MUTED)).padding([8, 10]);
            let settings = button(text("⚙ Settings").size(12).color(MUTED)).padding([8, 10]);

            if size.width < 560.0 {
                row![settings].align_y(Alignment::End).into()
            } else if size.width < 780.0 {
                row![collections, history, settings]
                    .spacing(4)
                    .align_y(Alignment::End)
                    .into()
            } else {
                row![collections, history, environments, settings]
                    .spacing(4)
                    .align_y(Alignment::End)
                    .into()
            }
        })
        .into();

        container(
            row![logo, horizontal(), nav_items]
                .spacing(12)
                .align_y(Alignment::Center)
                .padding([11, 18]),
        )
        .height(80) //TODO: i think it should be responsive
        .width(Length::Fill)
        .style(|_| container::background(SURFACE))
        .into()
    }
}
