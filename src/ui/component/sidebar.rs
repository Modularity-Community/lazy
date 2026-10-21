use iced::{
    Alignment, Background, Element, Length, Task, color,
    widget::{Space, button, column, container, row, space::horizontal, text, text_input},
};

use crate::ui::{
    style::button::LazyButton,
    theme::colors::{BG, BORDER, BTN_BG, CYAN, GREEN, MUTED, ORANGE, TEXT, VIOLET},
};

pub struct SidebarState {}
#[derive(Debug, Clone)]
pub enum SidebarMessage {}

impl SidebarState {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, _: SidebarMessage) -> Task<SidebarMessage> {
        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, SidebarMessage> {
        let header = row![
            column![
                text("Collections").size(15).color(TEXT),
                text("Your API workspaces").size(10).color(MUTED),
            ]
            .spacing(3),
            horizontal(),
            button(text("+").size(17).color(CYAN))
                .style(|_theme, _style| button::Style {
                    background: Background::Color(BTN_BG).into(),
                    border: iced::Border {
                        color: BORDER,
                        width: 1.0,
                        radius: 7.0.into(),
                    },
                    text_color: VIOLET,
                    ..Default::default()
                })
                .padding([3, 10]),
        ]
        .align_y(Alignment::Center);

        let search_bar = text_input("Search collections...", "")
            .padding([9, 10])
            .size(12)
            .style(|_theme, _status| iced::widget::text_input::Style {
                background: color!(0x1D2030).into(),
                border: iced::Border {
                    color: BORDER,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                icon: MUTED,
                placeholder: MUTED,
                value: TEXT,
                selection: VIOLET,
            });

        let collection = row![
            text("Auth API").size(13).color(TEXT),
            horizontal(),
            text("3").size(10).color(MUTED),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let requests = column![
            row![
                Space::new().width(15),
                text("POST").size(10).color(GREEN),
                text("Login").size(12).color(TEXT)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                Space::new().width(15),
                text("GET").size(10).color(CYAN),
                text("Profile").size(12).color(MUTED)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                Space::new().width(15),
                text("POST").size(10).color(GREEN),
                text("Refresh Token").size(12).color(MUTED)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(13);

        let other_collection = text("User Management").size(13).color(TEXT);

        let footer = column![
            button(text("Trash").size(12).color(MUTED))
                .btn_style()
                .padding([7, 8]),
            button(text("Collection settings").size(12).color(MUTED))
                .btn_style()
                .padding([7, 8]),
        ]
        .spacing(4);

        container(
            column![
                header,
                search_bar,
                Space::new().width(5),
                collection,
                requests,
                other_collection,
                horizontal(),
                footer
            ]
            .spacing(14)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fixed(232.0))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(BG.into()),
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
