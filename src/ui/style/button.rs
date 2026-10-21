use iced::{
    Background,
    widget::{Button, button::Style},
};

use crate::ui::theme::colors::{BORDER, BTN_BG, VIOLET};

pub trait LazyButton<'a, Message> {
    fn btn_style(self) -> Self;
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
}
