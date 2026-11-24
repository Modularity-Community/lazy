use iced::{
    Background, Border, Color, Element, Length,
    widget::{button, column, container, text},
};

use crate::ui::theme::colors::{MENU_BG, MENU_BORDER, MENU_HOVER, MENU_TEXT};

pub struct MenuItem<Message> {
    pub label: String,
    pub message: Message,
    pub destructive: bool,
}

impl<Message> MenuItem<Message> {
    pub fn new(label: impl Into<String>, message: Message) -> Self {
        Self {
            label: label.into(),
            message,
            destructive: false,
        }
    }

    // pub fn destructive(mut self) -> Self {
    //     self.destructive = true;
    //     self
    // }
}

pub fn menu<'a, Message: Clone + 'a>(items: Vec<MenuItem<Message>>) -> Element<'a, Message> {
    let mut content = column![].spacing(3);

    for item in items {
        let color = if item.destructive {
            Color::from_rgb8(255, 145, 155)
        } else {
            MENU_TEXT
        };

        let action = button(text(item.label).size(12).color(color))
            .width(Length::Fill)
            .padding([9, 12])
            .on_press(item.message)
            .style(move |_, status| {
                let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);

                button::Style {
                    background: hovered.then_some(Background::Color(MENU_HOVER)),
                    text_color: color,
                    border: Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

        content = content.push(action);
    }

    container(content)
        .width(Length::Fixed(190.0))
        .padding(5)
        .style(|_| container::Style {
            background: Some(Background::Color(MENU_BG)),
            border: Border {
                color: MENU_BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}
