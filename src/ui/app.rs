use iced::{Length, Task};

use iced::widget::{column, container, row};

use crate::ui::component::header_bar::{HeaderBarMessage, HeaderBarState};
use crate::ui::component::navbar::{NavbarMessage, NavbarState};
use crate::ui::component::request::{RequestPanelMessage, RequestPanelState};
use crate::ui::component::response::{ResponsePanelMessage, ResponsePanelState};
use crate::ui::component::sidebar::{SidebarMessage, SidebarState};
use iced::Element;

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
    Navbar(NavbarMessage),
    ExecuteRequest,
    RequestFinish(Result<String, String>),
}

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
            AppMessage::Navbar(msg) => self.navbar.update(msg).map(AppMessage::Navbar),
            AppMessage::Sidebar(msg) => self.sidebar.update(msg).map(AppMessage::Sidebar),
            AppMessage::Header(msg) => {
                if matches!(msg, HeaderBarMessage::SendReqeust) {
                    return Task::done(AppMessage::ExecuteRequest);
                }

                self.header_bar.update(msg).map(AppMessage::Header)
            }
            AppMessage::RequestPanel(msg) => {
                self.request_panel.update(msg).map(AppMessage::RequestPanel)
            }
            AppMessage::ResponsePanel(msg) => self
                .response_panel
                .update(msg)
                .map(AppMessage::ResponsePanel),
            AppMessage::ExecuteRequest => Task::none(),
            AppMessage::RequestFinish(_) => Task::none(),
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, AppMessage> {
        let sidebar_view = self.sidebar.view().map(AppMessage::Sidebar);
        let navbar_view = self.navbar.view().map(AppMessage::Navbar);
        let header_view = self.header_bar.view().map(AppMessage::Header);
        let request_view = self.request_panel.view().map(AppMessage::RequestPanel);
        let response_view = self.response_panel.view().map(AppMessage::ResponsePanel);

        let main_content = column![
            header_view,
            row![
                container(request_view).width(Length::FillPortion(1)),
                container(response_view).width(Length::FillPortion(1)),
            ]
            .height(Length::Fill)
            .spacing(10)
        ]
        .spacing(10);

        let body = row![
            container(sidebar_view).width(Length::Fixed(220.0)),
            container(main_content).width(Length::Fill),
        ]
        .spacing(10);

        column![
            container(navbar_view).width(Length::Fill),
            container(body).width(Length::Fill).height(Length::Fill)
        ]
        .into()
    }
}
