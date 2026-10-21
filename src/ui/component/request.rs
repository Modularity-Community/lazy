use crate::ui::theme::colors::{BORDER, CYAN, EDITOR_BG, MUTED, SURFACE, TEXT, VIOLET};
use iced::{
    Alignment, Element, Length, Task, color,
    highlighter::{Highlighter, Settings, Theme},
    widget::{
        button, column, container, row,
        space::horizontal,
        text,
        text_editor::{self, Edit},
    },
};
use std::sync::Arc;
pub struct RequestPanelState {
    pub body: text_editor::Content,
    pub is_menu_open: bool,
}

#[derive(Debug, Clone)]
pub enum RequestPanelMessage {
    BodyEdited(text_editor::Action),
    Prettify,
    ToggleMenu,
}

impl RequestPanelState {
    pub fn new() -> Self {
        Self {
            body: text_editor::Content::with_text(
                "{\n  \"name\": \"Lazy\",\n  \"enabled\": true\n}",
            ),
            is_menu_open: false,
        }
    }

    pub fn update(&mut self, message: RequestPanelMessage) -> Task<RequestPanelMessage> {
        match message {
            RequestPanelMessage::BodyEdited(action) => {
                self.body.perform(action);
            }
            RequestPanelMessage::Prettify => self.prettify_body(),
            RequestPanelMessage::ToggleMenu => self.is_menu_open = !self.is_menu_open,
        }
        Task::none()
    }

    fn prettify_body(&mut self) {
        let raw = self.body.text();
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                if pretty != raw {
                    self.body.perform(text_editor::Action::SelectAll);
                    self.body
                        .perform(text_editor::Action::Edit(Edit::Paste(Arc::new(pretty))));
                }
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, RequestPanelMessage> {
        let title = row![
            column![
                text("Request").size(16).color(TEXT),
                text("Configure the payload for your API call")
                    .size(11)
                    .color(MUTED),
            ]
            .spacing(3),
            horizontal(),
            text("BODY").size(10).color(VIOLET),
        ]
        .align_y(Alignment::Center);

        let tabs = row![
            text("Params").size(12).color(MUTED),
            text("Auth").size(12).color(MUTED),
            text("Headers").size(12).color(MUTED),
            container(text("Body").size(12).color(VIOLET))
                .padding([7, 10])
                .style(|_| container::Style {
                    background: Some(color!(0x302A48).into()),
                    border: iced::Border {
                        radius: 7.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let format_button = button(text("Format JSON").size(11).color(CYAN))
            .on_press(RequestPanelMessage::Prettify)
            .padding([6, 9])
            .style(|_theme, _status| iced::widget::button::Style {
                background: Some(color!(0x22283A).into()),
                text_color: CYAN,
                border: iced::Border {
                    color: BORDER,
                    width: 1.0,
                    radius: 7.0.into(),
                },
                ..Default::default()
            });

        let sub_tabs = row![
            text("JSON").size(11).color(CYAN),
            text("UTF-8").size(10).color(MUTED),
            horizontal(),
            format_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let editor = text_editor::TextEditor::new(&self.body)
            .height(Length::Fill)
            .on_action(RequestPanelMessage::BodyEdited)
            .highlight_with::<Highlighter>(
                Settings {
                    theme: Theme::Base16Ocean,
                    token: "json".to_string(),
                },
                |high, _theme| high.to_format(),
            );

        container(
            column![
                title,
                container(tabs).padding([8, 0]),
                container(sub_tabs).padding([4, 0]),
                container(editor)
                    .padding(12)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(EDITOR_BG.into()),
                        border: iced::Border {
                            color: BORDER,
                            width: 1.0,
                            radius: 10.0.into()
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(8)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(SURFACE.into()),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 13.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}
