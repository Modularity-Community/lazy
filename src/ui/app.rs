use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::widget::{
    button, column, container, responsive, row, scrollable, space::horizontal, text,
};
use iced::{Alignment, Element, Font, Length, Size, Subscription, Task, window};

use crate::domain::request_model::RequestSavedModel;
use crate::enums::auth_config::AuthConfig;
use crate::enums::method::HttpMethod;
use crate::infra::client::http_client::HttpClient;
use crate::infra::local_storage::conn::init_db;
use crate::infra::local_storage::sqlite_repo::{DBRepository, SqliteRepository};
use crate::ui::component::app_overlay::AppOverlay;
use crate::ui::component::header_bar::{HeaderBarEvent, HeaderBarState};
use crate::ui::component::navbar::{NavbarEvent, NavbarMessage};
use crate::ui::component::request::{RequestPanelEvent, RequestPanelState};
use crate::ui::component::response::{ResponsePanelEvent, ResponsePanelState};
use crate::ui::component::sidebar::{SidebarEvent, SidebarState};
use crate::ui::icon::icon;
use crate::ui::style::button::LazyButton;
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
    toasts: Vec<ToastNotification>,
    next_toast_id: u64,
    curl_preview: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone)]
struct ToastNotification {
    id: u64,
    title: String,
    description: String,
    kind: ToastKind,
}

/// snapshot from Send time, not mutable editor state at HTTP completion time
#[derive(Debug, Clone)]
pub struct RequestSnapshot {
    id: Option<i64>,
    folder_id: Option<i64>,
    selection_revision: u64,
    method: HttpMethod,
    url: String,
    body: String,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Sidebar(SidebarEvent),
    Header(HeaderBarEvent),
    RequestPanel(RequestPanelEvent),
    ResponsePanel(ResponsePanelEvent),
    Navbar(NavbarMessage),
    ExecuteRequest,
    RequestFinish {
        snapshot: RequestSnapshot,
        result: Result<(String, String, String), String>,
    },
    ToastExpired(u64),
    CloseToast(u64),
    CloseCurlPreview,
    CopyCurlPreview,
    ScreenSize(Option<Size<f32>>),

    DatabaseReady(Result<Arc<SqliteRepository>, String>),
    None,
}

fn same_selection(
    origin_revision: u64,
    expected_id: Option<i64>,
    current_revision: u64,
    active_id: Option<i64>,
) -> bool {
    origin_revision == current_revision && expected_id == active_id
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
            toasts: Vec::new(),
            next_toast_id: 0,
            curl_preview: None,
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

    fn push_toast(
        &mut self,
        kind: ToastKind,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Task<AppMessage> {
        let expiration = if matches!(kind, ToastKind::Error) {
            Duration::from_secs(7)
        } else {
            Duration::from_secs(4)
        };
        self.push_toast_for(kind, title, description, expiration)
    }

    // customize duration per notification without changing the overlay widget
    fn push_toast_for(
        &mut self,
        kind: ToastKind,
        title: impl Into<String>,
        description: impl Into<String>,
        expiration: Duration,
    ) -> Task<AppMessage> {
        self.next_toast_id = self.next_toast_id.wrapping_add(1);
        let id = self.next_toast_id;
        if self.toasts.len() >= 4 {
            self.toasts.remove(0);
        }
        self.toasts.push(ToastNotification {
            id,
            title: title.into(),
            description: description.into(),
            kind,
        });
        // capturing id makes late expiration safe after a manual dismissal
        Task::perform(
            async move {
                tokio::time::sleep(expiration).await;
                id
            },
            AppMessage::ToastExpired,
        )
    }

    fn toast_view<'a>(&'a self) -> Option<Element<'a, AppMessage>> {
        if self.toasts.is_empty() {
            return None;
        }
        let mut notifications = column![].spacing(9);
        for toast in &self.toasts {
            let accent = match toast.kind {
                ToastKind::Info => colors::CYAN,
                ToastKind::Success => colors::GREEN,
                ToastKind::Error => colors::RED,
            };
            let dismiss = button(icon("fa-close").size(20).color(colors::MUTED))
                .on_press(AppMessage::CloseToast(toast.id))
                .style(|_, _| button::Style::default())
                .padding([0, 6]);
            let card = container(
                row![
                    container(icon("fa-list-ol").size(11).color(accent)).padding([4, 0]),
                    column![
                        text(&toast.title).size(13).color(colors::TEXT),
                        text(&toast.description).size(11).color(colors::MUTED),
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                    dismiss,
                ]
                .align_y(Alignment::Start)
                .spacing(10),
            )
            .width(Length::Fill)
            .padding(13)
            .style(move |_| container::Style {
                background: Some(colors::SURFACE.into()),
                border: iced::Border {
                    color: accent,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            });
            notifications = notifications.push(card);
        }
        Some(container(notifications).width(Length::Fixed(345.0)).into())
    }

    fn curl_modal<'a>(&'a self) -> Option<Element<'a, AppMessage>> {
        let command = self.curl_preview.as_ref()?;
        let title = row![
            column![
                text("cURL Preview").size(18).color(colors::TEXT),
                text("POSIX shell (bash / zsh). Review secrets before sharing.")
                    .size(11)
                    .color(colors::MUTED),
            ]
            .spacing(6),
            horizontal(),
            button(icon("fa-close").size(22).color(colors::TEXT))
                .on_press(AppMessage::CloseCurlPreview)
                .style(|_, _| button::Style::default())
                .btn_style()
        ]
        .align_y(Alignment::Center);
        let code = container(
            scrollable(
                text(command)
                    .size(12)
                    .font(Font::with_name("Fira Code"))
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .width(Length::Fill)
                    .color(colors::CYAN),
            )
            .height(Length::Fixed(220.0)),
        )
        .width(Length::Fill)
        .padding(12)
        .style(|_| container::Style {
            background: Some(colors::EDITOR_BG.into()),
            border: iced::Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });
        let actions = row![
            horizontal(),
            button(text("Close"))
                .on_press(AppMessage::CloseCurlPreview)
                .padding([9, 16])
                .btn_style(),
            button(text("Copy cURL"))
                .on_press(AppMessage::CopyCurlPreview)
                .padding([9, 16])
                .btn_style(),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        Some(
            container(column![title, code, actions].spacing(18).padding(20))
                .width(Length::Fixed(650.0))
                .style(|_| container::Style {
                    background: Some(colors::SURFACE.into()),
                    border: iced::Border {
                        color: colors::VIOLET,
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                })
                .into(),
        )
    }

    pub fn update(&mut self, message: AppMessage, _now: Instant) -> Task<AppMessage> {
        match message {
            AppMessage::None => Task::none(),
            AppMessage::ToastExpired(id) | AppMessage::CloseToast(id) => {
                self.toasts.retain(|toast| toast.id != id);
                Task::none()
            }
            AppMessage::CloseCurlPreview => {
                self.curl_preview = None;
                Task::none()
            }
            AppMessage::CopyCurlPreview => match self.curl_preview.as_ref() {
                Some(command) => {
                    let command = command.clone();
                    let toast = self.push_toast(
                        ToastKind::Success,
                        "cURL",
                        "Copy command sent to clipboard",
                    );
                    Task::batch([iced::clipboard::write(command), toast])
                }
                None => Task::none(),
            },
            AppMessage::Sidebar(SidebarEvent::RenameFinished(result)) => {
                let task = self
                    .sidebar
                    .update(
                        SidebarEvent::RenameFinished(result.clone()),
                        self.repo.as_ref(),
                    )
                    .map(AppMessage::Sidebar);
                let toast = match result {
                    Ok(()) => {
                        self.push_toast(ToastKind::Success, "Name updated", "Saved to SQLite")
                    }
                    Err(err) => self.push_toast(ToastKind::Error, "Rename failed", err),
                };
                Task::batch([task, toast])
            }
            AppMessage::Sidebar(SidebarEvent::RequestCreated(result)) => {
                let task = self
                    .sidebar
                    .update(
                        SidebarEvent::RequestCreated(result.clone()),
                        self.repo.as_ref(),
                    )
                    .map(AppMessage::Sidebar);
                let toast = match result {
                    Ok(_) => self.push_toast(
                        ToastKind::Info,
                        "Request created",
                        "Ready to edit and send",
                    ),
                    Err(err) => self.push_toast(ToastKind::Error, "Create failed", err),
                };
                Task::batch([task, toast])
            }
            AppMessage::Sidebar(SidebarEvent::GenerateCurlReady(result)) => match result {
                Ok(command) => {
                    self.curl_preview = Some(command);
                    Task::none()
                }
                Err(err) => self.push_toast(ToastKind::Error, "Generate cURL failed", err),
            },
            AppMessage::Sidebar(SidebarEvent::CurrentRequestSaved {
                origin_revision,
                expected_id,
                result,
            }) => {
                self.header_bar.is_saving = false;
                match result {
                    Ok(id) => {
                        if same_selection(
                            origin_revision,
                            expected_id,
                            self.sidebar.selection_revision,
                            self.sidebar.active_request_id,
                        ) {
                            self.sidebar.active_request_id = Some(id);
                        }
                        // distinct from generic RequestSaved: INSERT returns the real ID
                        Task::done(AppMessage::Sidebar(SidebarEvent::ReloadWorkspace))
                    }
                    Err(err) => self.push_toast(ToastKind::Error, "Save request failed", err),
                }
            }
            AppMessage::DatabaseReady(Ok(repo)) => {
                tracing::info!("database ready");
                self.repo = Some(repo.clone());
                Task::done(AppMessage::Sidebar(SidebarEvent::ReloadWorkspace))
            }
            AppMessage::DatabaseReady(Err(err)) => {
                tracing::error!("failed to initialize db: {err}");
                Task::none()
            }
            AppMessage::Sidebar(SidebarEvent::SelectRequest(req)) => {
                if self.sidebar.active_request_id == req.id && req.id.is_some() {
                    // double-click generates two ordinary clicks before BeginRename
                    // do not discard unsaved editor changes when re-clicking the active row
                    self.sidebar.context_menu = None;
                    return Task::none();
                }
                self.sidebar.selection_revision = self.sidebar.selection_revision.wrapping_add(1);
                self.sidebar.context_menu = None;
                self.header_bar.url = req.url;
                self.header_bar.request_dir = req.name; //TODO: should append folder_name + request_name;
                self.header_bar.method = req.method.parse().unwrap_or(HttpMethod::Get);
                self.sidebar.active_request_id = req.id;
                self.sidebar.current_folder_id = req.folder_id;
                let task_response = self
                    .response_panel
                    .response_editor
                    .set_content(&req.response.unwrap_or_default());
                let task_request = self.request_panel.request_editor.set_content(&req.body);

                Task::batch(vec![task_response, task_request]).map(|_e| AppMessage::None)
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
            AppMessage::ResponsePanel(msg) => {
                if matches!(msg, ResponsePanelEvent::ResponseEditorAction(_)) {
                    self.request_panel.request_editor.editor.lose_focus();
                    return self
                        .response_panel
                        .update(msg)
                        .map(AppMessage::ResponsePanel);
                }
                self.response_panel
                    .update(msg)
                    .map(AppMessage::ResponsePanel)
            }
            AppMessage::ExecuteRequest => {
                if self.header_bar.is_sending || self.header_bar.is_saving {
                    return Task::none();
                }
                if self.repo.is_none() {
                    return self.push_toast(
                        ToastKind::Error,
                        "Database not ready",
                        "Cannot save the request yet",
                    );
                }
                self.header_bar.is_sending = true;
                let snapshot = RequestSnapshot {
                    id: self.sidebar.active_request_id,
                    folder_id: self.sidebar.current_folder_id,
                    selection_revision: self.sidebar.selection_revision,
                    method: self.header_bar.method,
                    url: self.header_bar.url.clone(),
                    body: self.request_panel.request_editor.editor.content(),
                };
                let client = self.http_client.clone();
                Task::perform(
                    async move {
                        let start_time = Instant::now();
                        let result = match snapshot.method {
                            HttpMethod::Get => client.get(&snapshot.url).await,
                            HttpMethod::Post => client.post(&snapshot.url, &snapshot.body).await,
                            HttpMethod::Put => client.put(&snapshot.url, &snapshot.body).await,
                            HttpMethod::Delete => {
                                client.delete(&snapshot.url, &snapshot.body).await
                            }
                            HttpMethod::Patch => client.patch(&snapshot.url, &snapshot.body).await,
                        };
                        let elapsed = start_time.elapsed();
                        let time_str = format!("{} ms", elapsed.as_millis());
                        let result = match result {
                            Ok(response) => {
                                let status_code = response.status().as_u16().to_string();
                                response
                                    .text()
                                    .await
                                    .map_err(|e| e.to_string())
                                    .map(|body| (body, status_code, time_str))
                            }
                            Err(err) => Err(err.to_string()),
                        };
                        (snapshot, result)
                    },
                    |(snapshot, result)| AppMessage::RequestFinish { snapshot, result },
                )
            }
            AppMessage::RequestFinish { snapshot, result } => {
                self.header_bar.is_sending = false;
                let same_selection = same_selection(
                    snapshot.selection_revision,
                    snapshot.id,
                    self.sidebar.selection_revision,
                    self.sidebar.active_request_id,
                );
                let response_task = if same_selection {
                    self.response_panel
                        .update(ResponsePanelEvent::ResponseReceived(result.clone()))
                        .map(AppMessage::ResponsePanel)
                } else {
                    Task::none()
                };

                match result {
                    Ok((body, _status, _elapsed)) => {
                        // keep Send disabled until INSERT/UPDATE returns, this also
                        // prevents a second INSERT before the new ID is available
                        self.header_bar.is_saving = true;
                        let request = RequestSavedModel {
                            id: snapshot.id,
                            folder_id: snapshot.folder_id,
                            name: "New Request".to_string(),
                            method: snapshot.method.to_string(),
                            url: snapshot.url,
                            headers: vec![],
                            params: vec![],
                            auth: AuthConfig::None,
                            body: snapshot.body,
                            response: Some(body), // actual HTTP body, not the previous editor contents
                        };
                        Task::batch([
                            response_task,
                            Task::done(AppMessage::Sidebar(SidebarEvent::SaveCurrentRequest {
                                request,
                                origin_revision: snapshot.selection_revision,
                            })),
                        ])
                    }
                    Err(err) => Task::batch([
                        response_task,
                        self.push_toast(ToastKind::Error, "HTTP request failed", err),
                    ]),
                }
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, AppMessage> {
        let main_content = responsive(move |size| {
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
                let main_area = container(
                    column![header, panels]
                        .spacing(14)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill);

                if self.navbar.show_workspace {
                    let sidebar = self.sidebar.view().map(AppMessage::Sidebar);
                    row![
                        container(sidebar)
                            .width(Length::Fixed(232.0))
                            .height(Length::Fill),
                        main_area,
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    main_area.into()
                }
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
        });
        AppOverlay::new(
            main_content,
            self.toast_view(),
            self.curl_modal(),
            AppMessage::CloseCurlPreview,
        )
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_id_is_adopted_only_by_original_editor_selection() {
        assert!(same_selection(9, None, 9, None));
        assert!(!same_selection(9, None, 10, None));
        assert!(!same_selection(9, Some(3), 9, Some(4)));
        assert!(same_selection(9, Some(3), 9, Some(3)));
    }
}
