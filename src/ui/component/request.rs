use std::sync::Arc;

use iced::{
    Element, Length, Task,
    highlighter::{Highlighter, Settings, Theme},
    widget::{
        column, container, horizontal_space, mouse_area, row, text,
        text_editor::{self, Edit},
    },
};

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
            body: text_editor::Content::new(),
            is_menu_open: false,
        }
    }

    pub fn update(&mut self, message: RequestPanelMessage) -> Task<RequestPanelMessage> {
        match message {
            RequestPanelMessage::BodyEdited(action) => {
                self.body.perform(action);
                Task::none()
            }
            RequestPanelMessage::Prettify => {
                //TODO: Action prettify json
                Task::none()
            }
            RequestPanelMessage::ToggleMenu => {
                tracing::info!("Toggle Menu Opened");
                self.is_menu_open = !self.is_menu_open;

                let txt_body = self.body.text();

                if !txt_body.trim().is_empty() {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&txt_body) {
                        if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                            if pretty_json != txt_body {
                                self.body.perform(text_editor::Action::SelectAll);
                                self.body.perform(text_editor::Action::Edit(Edit::Paste(
                                    Arc::new(pretty_json),
                                )));
                            }
                        }
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, RequestPanelMessage> {
        let tabs = row![
            text("Params").size(14),
            text("Auth").size(14),
            text("Headers 4").size(14),
            text("Body").size(14),
        ]
        .spacing(20);

        let sub_tabs = row![
            text("JSON").size(13),
            text("Form").size(13),
            text("Text").size(13),
            text("GraphQL").size(13),
            horizontal_space(),
        ]
        .spacing(15)
        .padding([10, 0]);

        let editor = iced::widget::text_editor(&self.body)
            .height(Length::Fill)
            .on_action(RequestPanelMessage::BodyEdited)
            .highlight_with::<Highlighter>(
                Settings {
                    theme: Theme::SolarizedDark,
                    token: "json".to_string(),
                },
                |high, _theme| high.to_format(),
            );

        let editor_area = mouse_area(editor).on_right_press(RequestPanelMessage::ToggleMenu);

        container(column![tabs, sub_tabs, editor_area].spacing(10).padding(15))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
