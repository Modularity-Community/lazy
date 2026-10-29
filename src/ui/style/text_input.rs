use iced::{
    Renderer, Theme,
    widget::{TextInput, text_input::Style},
};

use crate::ui::theme::colors::{BORDER, MUTED, SURFACE, TEXT, VIOLET};

pub trait LazyTextInput<'a, Message> {
    fn search_style(self) -> Self;
}

impl<'a, Message: Clone> LazyTextInput<'a, Message> for TextInput<'a, Message, Theme, Renderer> {
    fn search_style(self) -> Self {
        self.style(|_theme, _status| Style {
            background: SURFACE.into(),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 9.0.into(),
            },
            icon: MUTED,
            placeholder: MUTED,
            value: TEXT,
            selection: VIOLET,
        })
    }
}
