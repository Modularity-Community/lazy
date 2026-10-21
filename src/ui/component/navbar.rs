use iced::{
    Alignment, Background, Element, Length, Size, Task,
    widget::{button, container, responsive, row, space::horizontal, text},
};

use crate::ui::{
    style::button::LazyButton,
    theme::colors::{BORDER, BTN_BG, MUTED, SURFACE, TEXT, VIOLET},
};
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

    pub fn view<'a>(&'a self, screen_size: &Option<Size<f32>>) -> Element<'a, NavbarMessage> {
        let logo = row![
            text("Lazy").size(21).color(TEXT),
            text("BETA").size(5).color(VIOLET),
        ]
        .spacing(7)
        .align_y(Alignment::Center);

        let window_width = screen_size.map(|s| s.width).unwrap_or(800.0);

        let nav_items: Element<'a, NavbarMessage> = {
            let collections = button(text("Collections").size(12))
                .btn_style()
                .padding([8, 10]);
            let history = button(text("History").size(12))
                .btn_style()
                .padding([8, 10]);
            let environments = button(text("Environments").size(12))
                .btn_style()
                .padding([8, 10]);
            let settings = button(text("Settings").size(12))
                .btn_style()
                .padding([8, 10]);

            if window_width < 560.0 {
                row![settings].align_y(Alignment::End).into()
            } else if window_width < 780.0 {
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
        };

        container(
            row![logo, horizontal(), nav_items]
                .spacing(12)
                .align_y(Alignment::Center)
                .padding([11, 18]),
        )
        .height(Length::Shrink)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(SURFACE.into()),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}
