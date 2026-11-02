use std::sync::Arc;
use std::time::Instant;

use iced::widget::{column, container, responsive, row, scrollable};
use iced::{Element, Length, Size, Subscription, Task, window};

use crate::domain::request_model::RequestSavedModel;
use crate::enums::auth_config::AuthConfig;
use crate::enums::method::HttpMethod;
use crate::infra::client::http_client::HttpClient;
use crate::infra::local_storage::conn::init_db;
use crate::infra::local_storage::sqlite_repo::{DBRepository, SqliteRepository};
use crate::ui::component::header_bar::{HeaderBarEvent, HeaderBarState};
use crate::ui::component::navbar::{NavbarEvent, NavbarMessage};
use crate::ui::component::request::{RequestPanelEvent, RequestPanelState};
use crate::ui::component::response::{ResponsePanelEvent, ResponsePanelState};
use crate::ui::component::sidebar::{SidebarEvent, SidebarState};
use crate::ui::theme::colors;

pub struct AppState {
    pub sidebar: SidebarState,
    pub header_bar: HeaderBarState,
    pub request_panel: RequestPanelState,
    pub response_panel: ResponsePanelState,
    pub navbar: NavbarEvent,
    pub screen_size: Option<Size<f32>>,
    pub http_client: HttpClient,
    pub repo: Option<Arc<SqliteRepository>>,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Sidebar(SidebarEvent),
    Header(HeaderBarEvent),
    RequestPanel(RequestPanelEvent),
    ResponsePanel(ResponsePanelEvent),
    Navbar(NavbarMessage),
    ExecuteRequest,
    RequestFinish(Result<(String, String, String), String>),
    ScreenSize(Option<Size<f32>>),

    DatabaseReady(Result<Arc<SqliteRepository>, String>),
    None,
}

impl AppState {
    pub fn new() -> (Self, Task<AppMessage>) {
        let state = Self {
            sidebar: SidebarState::new(),
            header_bar: HeaderBarState::new(),
            request_panel: RequestPanelState::new(),
            response_panel: ResponsePanelState::new(),
            navbar: NavbarEvent::new(),
            screen_size: None,
            http_client: HttpClient::new(),
            repo: None,
        };

        let init_task = Task::perform(
            async {
                let pool = init_db().await.map_err(|e| e.to_string());
                match pool {
                    Ok(p) => {
                        let repo = SqliteRepository::new(p);
                        repo.init_db().await?;
                        Ok(Arc::new(repo))
                    }
                    Err(err) => {
                        tracing::error!("failed init app, because pooling db err : {err}");
                        Err(err)
                    }
                }
            },
            AppMessage::DatabaseReady,
        );

        (state, init_task)
    }

    pub fn subscription(&self) -> Subscription<AppMessage> {
        window::resize_events().map(|(_id, size)| AppMessage::ScreenSize(Some(size)))
    }

    pub fn update(&mut self, message: AppMessage, _now: Instant) -> Task<AppMessage> {
        match message {
            AppMessage::None => Task::none(),
            AppMessage::DatabaseReady(Ok(repo)) => {
                tracing::info!("database ready");
                self.repo = Some(repo.clone());
                Task::none()
            }
            AppMessage::DatabaseReady(Err(err)) => {
                tracing::error!("failed to initialize db: {err}");
                Task::none()
            }
            // AppMessage::RequestSaved(Ok(inserted_id)) => {
            //     self.active_request_id = Some(inserted_id);
            //     tracing::info!("saved request id {inserted_id}");
            //     if let Some(repo) = self.repo.clone() {
            //         Task::perform(
            //             async move { repo.fetch_workspace_tree().await },
            //             AppMessage::WorkspaceLoaded,
            //         )
            //     } else {
            //         Task::none()
            //     }
            // }
            // AppMessage::RequestSaved(Err(err)) => {
            //     tracing::error!("failed to load workspace : {err}");
            //     Task::none()
            // }
            AppMessage::Sidebar(SidebarEvent::SelectRequest(req)) => {
                self.header_bar.url = req.url;
                self.header_bar.method = req.method.parse().unwrap_or(HttpMethod::Get);
                self.request_panel
                    .request_editor
                    .set_content(&req.body)
                    .map(RequestPanelEvent::BodyEdited)
                    .map(AppMessage::RequestPanel)
            }
            AppMessage::Sidebar(SidebarEvent::CreateFolder(parent_id)) => {
                // if let Some(repo) = self.repo.clone() {
                //     Task::perform(
                //         async move {
                //             repo.save_folder("New Folder", parent_id).await?;
                //             repo.fetch_workspace_tree().await
                //         },
                //         AppMessage::WorkspaceLoaded,
                //     )
                // } else {
                //     Task::none()
                // }
                Task::none()
            }
            AppMessage::ScreenSize(size) => {
                self.screen_size = size;
                Task::none()
            }
            AppMessage::Navbar(msg) => self.navbar.update(msg).map(AppMessage::Navbar),
            AppMessage::Sidebar(msg) => self
                .sidebar
                .update(msg, self.repo.as_ref())
                .map(AppMessage::Sidebar),
            AppMessage::Header(msg) => {
                if matches!(msg, HeaderBarEvent::SendReqeust) {
                    return Task::done(AppMessage::ExecuteRequest);
                }
                if matches!(msg, HeaderBarEvent::UrlEdited(_)) {
                    self.response_panel.response_editor.editor.lose_focus();
                    self.request_panel.request_editor.editor.lose_focus();
                    return self.header_bar.update(msg).map(AppMessage::Header);
                }
                self.header_bar.update(msg).map(AppMessage::Header)
            }
            AppMessage::RequestPanel(msg) => {
                if matches!(msg, RequestPanelEvent::BodyEdited(_)) {
                    self.response_panel.response_editor.editor.lose_focus();
                    return self.request_panel.update(msg).map(AppMessage::RequestPanel);
                }
                self.request_panel.update(msg).map(AppMessage::RequestPanel)
            }
            AppMessage::ResponsePanel(msg) => self
                .response_panel
                .update(msg)
                .map(AppMessage::ResponsePanel),
            AppMessage::ExecuteRequest => {
                self.header_bar.is_sending = true;
                let url = self.header_bar.url.clone();
                let method = self.header_bar.method;
                let client = self.http_client.clone();
                let request = self.request_panel.request_editor.editor.content();

                Task::perform(
                    async move {
                        let start_time = Instant::now();
                        let result = match method {
                            HttpMethod::Get => client.get(&url).await,
                            HttpMethod::Post => client.post(&url, &request).await,
                            HttpMethod::Put => client.put(&url, &request).await,
                            HttpMethod::Delete => client.delete(&url, &request).await,
                            HttpMethod::Patch => client.patch(&url, &request).await,
                        };
                        let elapsed = start_time.elapsed();
                        let time_str = format!("{} ms", elapsed.as_millis());
                        match result {
                            Ok(response) => {
                                let status_code = response.status().as_u16().to_string();
                                let body = response.text().await.unwrap_or_else(|e| e.to_string());
                                Ok((body, status_code, time_str))
                            }
                            Err(err) => Err(err.to_string()),
                        }
                    },
                    |e| AppMessage::RequestFinish(e),
                )
            }
            AppMessage::RequestFinish(result) => {
                let is_success = result.is_ok();

                let first_task = self.header_bar.update(HeaderBarEvent::FinishedRequest);
                let second_task = self
                    .response_panel
                    .update(ResponsePanelEvent::ResponseReceived(result));

                let mut tasks = vec![
                    first_task.map(AppMessage::Header),
                    second_task.map(AppMessage::ResponsePanel),
                ];

                tracing::info!("result success {is_success}");
                if is_success {
                    //     tasks.push(Task::done(AppMessage::SaveCurrentRequest));
                    let req_model = RequestSavedModel {
                        id: None,
                        folder_id: None,
                        name: "New Request".to_string(),
                        method: self.header_bar.method.to_string(),
                        url: self.header_bar.url.clone(),
                        headers: vec![],
                        params: vec![],
                        auth: AuthConfig::None,
                        body: self.request_panel.request_editor.editor.content(),
                    };

                    tasks.push(Task::done(AppMessage::Sidebar(
                        SidebarEvent::CreateRequest(req_model),
                    )));
                }

                Task::batch(tasks)
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, AppMessage> {
        responsive(move |size| {
            let navbar = self.navbar.view(&self.screen_size).map(AppMessage::Navbar);
            let header = self
                .header_bar
                .view(&self.screen_size)
                .map(AppMessage::Header);
            let request = self.request_panel.view().map(AppMessage::RequestPanel);
            let response = self.response_panel.view().map(AppMessage::ResponsePanel);

            let panels: Element<'a, AppMessage> = if size.width < 950.0 {
                column![
                    container(request)
                        .width(Length::Fill)
                        .height(Length::Fixed(390.0)),
                    container(response)
                        .width(Length::Fill)
                        .height(Length::Fixed(390.0)),
                ]
                .spacing(7)
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
                .spacing(7)
                .width(Length::Fill)
                .height(Length::Fill)
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
                    container(
                        column![header, panels]
                            .spacing(14)
                            .width(Length::Fill)
                            .height(Length::Fill)
                    )
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
            .style(|_| container::background(colors::BG))
            .into()
        })
        .into()
    }
}
