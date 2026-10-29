use iced::{
    Background,
    widget::{Button, button::Style},
};

use crate::ui::theme::colors::{BG, BORDER, BTN_BG, CYAN, HOVERED, VIOLET};

pub trait LazyButton<'a, Message> {
    fn btn_style(self) -> Self;
    fn btn_execute_style(self) -> Self;
}

impl<'a, Message> LazyButton<'a, Message> for Button<'a, Message> {
    fn btn_style(self) -> Self {
        self.style(|_theme, _status| Style {
            background: Background::Color(BTN_BG).into(),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 7.0.into(),
            },
            text_color: VIOLET,
            ..Default::default()
        })
        .padding([8, 10])
    }

    fn btn_execute_style(self) -> Self {
        self.style(|_theme, status| {
            let background = match status {
                iced::widget::button::Status::Hovered => HOVERED,
                _ => CYAN,
            };
            Style {
                background: Some(background.into()),
                text_color: BG,
                border: iced::Border {
                    radius: 9.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
    }
}
