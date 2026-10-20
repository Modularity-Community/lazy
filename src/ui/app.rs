use iced::widget::{column, container, responsive, row, scrollable};
use iced::{Element, Length, Task, color};

use crate::ui::component::header_bar::{HeaderBarMessage, HeaderBarState};
use crate::ui::component::navbar::{NavbarMessage, NavbarState};
use crate::ui::component::request::{RequestPanelMessage, RequestPanelState};
use crate::ui::component::response::{ResponsePanelMessage, ResponsePanelState};
use crate::ui::component::sidebar::{SidebarMessage, SidebarState};

const BG: iced::Color = color!(0x11131D);
const WORKSPACE: iced::Color = color!(0x191C29);

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
        responsive(move |size| {
            let navbar = self.navbar.view().map(AppMessage::Navbar);
            let header = self.header_bar.view().map(AppMessage::Header);
            let request = self.request_panel.view().map(AppMessage::RequestPanel);
            let response = self.response_panel.view().map(AppMessage::ResponsePanel);

            let panels: Element<'a, AppMessage> = if size.width < 1180.0 {
                column![
                    container(request)
                        .width(Length::Fill)
                        .height(Length::Fixed(390.0)),
                    container(response)
                        .width(Length::Fill)
                        .height(Length::Fixed(390.0)),
                ]
                .spacing(14)
                .width(Length::Fill)
                .into()
            } else {
                row![
                    container(request)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill),
                    container(response)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill),
                ]
                .spacing(14)
                .height(Length::Fill)
                .width(Length::Fill)
                .into()
            };

            let content: Element<'a, AppMessage> = if size.width < 760.0 {
                column![
                    header,
                    scrollable(panels).height(Length::Fill).width(Length::Fill),
                ]
                .spacing(14)
                .padding(12)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                let sidebar = self.sidebar.view().map(AppMessage::Sidebar);
                row![
                    container(sidebar)
                        .width(Length::Fixed(232.0))
                        .height(Length::Fill),
                    column![header, panels]
                        .spacing(14)
                        .padding(16)
                        .width(Length::Fill)
                        .height(Length::Fill),
                ]
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            };

            container(
                column![
                    navbar,
                    container(content).width(Length::Fill).height(Length::Fill)
                ]
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::background(BG))
            .into()
        })
        .into()
    }
}
