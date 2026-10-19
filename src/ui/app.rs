use iced::Task;

use iced::widget::{Text, column, container, row};

use super::component::{header_bar, navbar, sidebar};
use crate::ui::component::header_bar::{HeaderBarMessage, HeaderBarState};
use crate::ui::component::navbar::NavbarState;
use crate::ui::component::request::{RequestPanelMessage, RequestPanelState};
use crate::ui::component::response::{ResponsePanelMessage, ResponsePanelState};
use crate::ui::component::sidebar::{SidebarMessage, SidebarState};
use iced::{Element, Length};

pub struct AppState {
    pub is_loading: bool,
    pub sidebar: SidebarState,
    pub header_bar: HeaderBarState,
    pub request_panel: RequestPanelState,
    pub response_panel: ResponsePanelState,
    pub navbar: NavbarState,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Sidebar(SidebarMessage),
    Header(HeaderBarMessage),
    RequestPanel(RequestPanelMessage),
    ResponsePanel(ResponsePanelMessage),
    ExecuteRequest,
    RequestFinish(Result<String, String>),
}

// url: String::from("https://jsonplaceholder.typicode.com/posts"),
// method: HttpMethod::Post,
// request_body: text_editor::Content::with_text("{}"),
// response_text: text_editor::Content::new(),
// status_text: String::from("-"),
// time_text: String::from("-"),
// is_loading: false,
impl AppState {
    pub fn new() -> (Self, Task<AppMessage>) {
        (
            Self {
                is_loading: false,
                sidebar: SidebarState::new(),
                header_bar: HeaderBarState::new(),
                request_panel: RequestPanelState::new(),
                response_panel: ResponsePanelState::new(),
                navbar: NavbarState::new(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::Sidebar(msg) => Task::none(),
            AppMessage::Header(msg) => {
                if matches!(msg, HeaderBarMessage::SendReqeust) {
                    return Task::done(AppMessage::ExecuteRequest);
                }

                self.header_bar.update(msg).map(AppMessage::Header)
            }
            AppMessage::RequestPanel(msg) => Task::none(),
            AppMessage::ResponsePanel(msg) => Task::none(),
            AppMessage::ExecuteRequest => Task::none(),
            AppMessage::RequestFinish(msg) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        // let content = column![
        //     navbar::view(),
        //     row![sidebar::view(), header_bar::view(self)].height(Length::Fill)
        // ];

        // container(content)
        //     .width(Length::Fill)
        //     .height(Length::Fill)
        //     .into()
        Text::new("test").into()
    }
}
