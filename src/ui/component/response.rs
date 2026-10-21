use crate::util;
use iced::{
    Alignment, Element, Length, Padding, Task, color,
    highlighter::{self, Highlighter},
    widget::{
        Space, TextEditor, column, container, row,
        space::horizontal,
        text,
        text_editor::{self, Action, Content, Edit, Motion},
    },
};

use crate::ui::theme::colors::{BORDER, CYAN, EDITOR_BG, GREEN, MUTED, RED, SURFACE, TEXT, VIOLET};

pub struct ResponsePanelState {
    pub status_text: String,
    pub time_text: String,
    pub response_text: Content,
}

#[derive(Debug, Clone)]
pub enum ResponsePanelMessage {
    ResponseEditorAction(Action),
    ResponseEditorRightClick,
    ResponseReceived(Result<(String, String, String), String>),
}

impl ResponsePanelState {
    pub fn new() -> Self {
        Self {
            status_text: String::from("Waiting"),
            time_text: String::new(),
            response_text: Content::with_text(
                "// Response data will appear here after you send a request",
            ),
        }
    }

    pub fn update(&mut self, msg: ResponsePanelMessage) -> Task<ResponsePanelMessage> {
        match msg {
            ResponsePanelMessage::ResponseEditorRightClick => Task::none(),
            ResponsePanelMessage::ResponseEditorAction(action) => {
                if !action.is_edit() {
                    self.response_text.perform(action);
                }
                Task::none()
            }
            ResponsePanelMessage::ResponseReceived(result) => {
                match result {
                    Ok((body, status, time)) => {
                        let body = util::make_json_pretty(&body);
                        self.response_text = Content::with_text(&body);
                        self.status_text = status;
                        self.time_text = time;

                        self.response_text
                            .perform(Action::Move(Motion::DocumentStart));

                        self.response_text.perform(Action::Edit(Edit::Backspace));
                    }
                    Err(err) => {
                        self.response_text =
                            Content::with_text(&format!("Request failed\n\n{err}"));
                        self.status_text = String::from("Error");
                        self.time_text.clear();
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, ResponsePanelMessage> {
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
                        radius: 7.0.into()
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

        let editor = TextEditor::new(&self.response_text)
            .height(Length::Fill)
            .on_action(ResponsePanelMessage::ResponseEditorAction)
            .style(|_theme, _status| text_editor::Style {
                background: EDITOR_BG.into(),
                border: iced::Border {
                    color: BORDER,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                placeholder: MUTED,
                value: TEXT,
                selection: MUTED,
            })
            .highlight_with::<Highlighter>(
                highlighter::Settings {
                    theme: highlighter::Theme::Base16Ocean,
                    token: "json".to_string(),
                },
                |high, _theme| high.to_format(),
            );

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
                radius: 13.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}
