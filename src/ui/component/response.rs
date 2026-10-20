use iced::{
    Element, Length, Task, color,
    highlighter::{self, Highlighter},
    widget::{
        Space, TextEditor, column, container, horizontal_space, mouse_area, row, text,
        text_editor::{Action, Content, Edit, Motion},
    },
};

use crate::util;

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
            status_text: String::from("-"),
            time_text: String::from("-"),
            response_text: Content::new(),
        }
    }

    pub fn update(&mut self, msg: ResponsePanelMessage) -> Task<ResponsePanelMessage> {
        match msg {
            ResponsePanelMessage::ResponseEditorRightClick => {
                tracing::info!("Response on right click");
                Task::none()
            }
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

                        self.response_text
                            .perform(Action::Move(Motion::DocumentStart));

                        self.response_text.perform(Action::Edit(Edit::Backspace));

                        self.status_text = status;
                        self.time_text = time;
                    }
                    Err(err) => {
                        self.response_text = Content::with_text(&format!("Error: {}", err));
                        self.status_text = String::from("Error");
                    }
                };

                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, ResponsePanelMessage> {
        let header = row![
            text("Response").size(16),
            horizontal_space(),
            text(format!("{}", &self.status_text)).size(12),
            Space::with_width(10),
            text(format!("{}", &self.time_text)).size(12),
        ]
        .align_y(iced::Alignment::Center);

        //TODO: Count headers ambil dari response headers
        //TODO:  Cookies ambil dari response
        let tabs = row![
            text("Body").size(14),
            text("Headers 12").size(14),
            text("Cookies 2").size(14),
        ]
        .spacing(20);

        let sub_tabs = row![
            text("Pretty").size(13),
            text("Raw").size(13),
            text("Preview").size(13),
            horizontal_space(),
            text("Search...").size(13),
        ]
        .spacing(15)
        .padding([10, 0]);

        let editor = TextEditor::new(&self.response_text)
            .height(Length::Fill)
            .on_action(ResponsePanelMessage::ResponseEditorAction)
            .highlight_with::<Highlighter>(
                highlighter::Settings {
                    theme: highlighter::Theme::Base16Ocean,
                    token: "json".to_string(),
                },
                |high, _theme| high.to_format(),
            );

        container(
            mouse_area(
                column![header, Space::with_height(10), tabs, sub_tabs, editor]
                    .spacing(10)
                    .padding(15),
            )
            .on_right_press(ResponsePanelMessage::ResponseEditorRightClick),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::background(color!(0x121212)))
        .into()
    }
}
