use crate::ui::{
    component::json_editor::JsonEditor,
    theme::colors::{BORDER, BTN_BG, CYAN, MUTED, SURFACE, TEXT, VIOLET},
};
use iced::{
    Alignment, Element, Length, Task,
    widget::{Container, button, column, container, row, space::horizontal, text},
};
use iced_code_editor::Message as EditorMsg;

pub struct RequestPanelState {
    pub request_editor: JsonEditor,
}

#[derive(Debug, Clone)]
pub enum RequestPanelEvent {
    BodyEdited(EditorMsg),
    Prettify,
}

impl RequestPanelState {
    pub fn new() -> Self {
        Self {
            request_editor: JsonEditor::new(""),
        }
    }

    pub fn update(&mut self, message: RequestPanelEvent) -> Task<RequestPanelEvent> {
        match message {
            RequestPanelEvent::BodyEdited(event) => {
                self.request_editor.validate();
                self.request_editor
                    .editor
                    .update(&event)
                    .map(RequestPanelEvent::BodyEdited)
            }
            RequestPanelEvent::Prettify => {
                self.request_editor.validate();
                self.request_editor.editor.lose_focus();
                self.request_editor
                    .format()
                    .map(RequestPanelEvent::BodyEdited)
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, RequestPanelEvent> {
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
                    background: Some(BTN_BG.into()),
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
            .on_press(RequestPanelEvent::Prettify)
            .padding([6, 9])
            .style(|_theme, _status| iced::widget::button::Style {
                background: Some(SURFACE.into()),
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

        let editor = Container::new(
            self.request_editor
                .view()
                .map(RequestPanelEvent::BodyEdited),
        )
        .height(Length::Fill);

        container(
            column![
                title,
                container(tabs).padding([8, 0]),
                container(sub_tabs).padding([4, 0]),
                editor,
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
                radius: 1.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}
