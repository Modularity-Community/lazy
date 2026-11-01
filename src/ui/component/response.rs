use crate::ui::component::json_editor::JsonEditor;
use iced::{
    Alignment, Element, Length, Padding, Task, color,
    widget::{Container, Space, column, container, row, space::horizontal, text},
};

use iced_code_editor::Message as EditorMsg;

use crate::ui::theme::colors::{BORDER, CYAN, GREEN, MUTED, RED, SURFACE, TEXT, VIOLET};

pub struct ResponsePanelState {
    pub status_text: String,
    pub time_text: String,
    pub response_editor: JsonEditor,
}

#[derive(Debug, Clone)]
pub enum ResponsePanelEvent {
    ResponseEditorAction(EditorMsg),
    ResponseReceived(Result<(String, String, String), String>),
}

impl ResponsePanelState {
    pub fn new() -> Self {
        Self {
            status_text: String::from("Waiting"),
            time_text: String::new(),
            response_editor: JsonEditor::new(
                "// Response data will appear here after you send a request",
            ),
        }
    }

    pub fn update(&mut self, msg: ResponsePanelEvent) -> Task<ResponsePanelEvent> {
        match msg {
            ResponsePanelEvent::ResponseEditorAction(action) => match action {
                EditorMsg::CharacterInput(_) => Task::none(),
                EditorMsg::Delete | EditorMsg::DeleteSelection => Task::none(),
                EditorMsg::Backspace => self
                    .response_editor
                    .editor
                    .update(&EditorMsg::ArrowKey(
                        iced_code_editor::ArrowDirection::Left,
                        false,
                    ))
                    .map(ResponsePanelEvent::ResponseEditorAction),
                _other => self
                    .response_editor
                    .editor
                    .update(&_other)
                    .map(ResponsePanelEvent::ResponseEditorAction),
            },
            ResponsePanelEvent::ResponseReceived(result) => match result {
                Ok((body, status, time)) => {
                    self.status_text = status;
                    self.time_text = time;

                    self.response_editor
                        .set_content(&body)
                        .map(ResponsePanelEvent::ResponseEditorAction)
                }
                Err(err) => {
                    self.status_text = String::from("Error");
                    self.time_text.clear();
                    self.response_editor
                        .set_content(&format!("Request failed\n\n{err}"))
                        .map(ResponsePanelEvent::ResponseEditorAction)
                }
            },
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, ResponsePanelEvent> {
        let status_color = if self.status_text.starts_with('2') {
            GREEN
        } else if self.status_text == "Error" {
            RED
        } else if self.status_text == "Waiting" {
            MUTED
        } else {
            VIOLET
        };

        let header = row![
            column![
                text("Response").size(16).color(TEXT),
                text("Inspect status and returned data")
                    .size(11)
                    .color(MUTED),
            ]
            .spacing(3),
            horizontal(),
            container(text(&self.status_text).size(11).color(status_color))
                .padding([5, 8])
                .style(move |_| container::Style {
                    background: Some(color!(0x292D40).into()),
                    border: iced::Border {
                        color: status_color,
                        width: 1.0,
                        radius: 1.0.into()
                    },
                    ..Default::default()
                }),
            Space::new().width(6),
            text(&self.time_text).size(11).color(MUTED),
        ]
        .align_y(Alignment::Center);

        let tabs = row![
            text("Body").size(12).color(VIOLET),
            text("Headers").size(12).color(MUTED),
            text("Cookies").size(12).color(MUTED),
            horizontal(),
            text("JSON").size(10).color(CYAN),
        ]
        .spacing(14)
        .align_y(Alignment::Center);

        let editor = Container::new(
            self.response_editor
                .view()
                .map(ResponsePanelEvent::ResponseEditorAction),
        )
        .height(Length::Fill);

        container(
            column![
                header,
                container(tabs).padding(Padding {
                    top: 13.0,
                    bottom: 0.0,
                    left: 8.0,
                    right: 0.0
                }),
                editor
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
