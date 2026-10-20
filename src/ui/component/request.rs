use std::sync::Arc;

use crate::util;
use iced::{
    Element, Length, Task,
    widget::{
        column, container, horizontal_space, mouse_area, row, text,
        text_editor::{self, Action, Edit},
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

                let txt_body = self.body.text();
                let body = util::make_json_pretty(&txt_body);
                self.body.perform(Action::Edit(Edit::Paste(Arc::new(body))));

                self.body.perform(text_editor::Action::Move(
                    text_editor::Motion::DocumentStart,
                ));

                self.body
                    .perform(text_editor::Action::Edit(Edit::Backspace));
                Task::none()
            }
            RequestPanelMessage::Prettify => Task::none(),
            RequestPanelMessage::ToggleMenu => {
                tracing::info!("Toggle Menu Opened");
                self.is_menu_open = !self.is_menu_open;
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
            .on_action(RequestPanelMessage::BodyEdited);

        let editor_area = mouse_area(editor).on_right_press(RequestPanelMessage::ToggleMenu);

        container(column![tabs, sub_tabs, editor_area].spacing(10).padding(15))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
