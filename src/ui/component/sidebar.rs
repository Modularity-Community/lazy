use std::sync::Arc;

use iced::{
    Alignment, Background, Element, Length, Point, Task,
    widget::{
        Space, button, column, container, mouse_area, row, space::horizontal, text, text_input,
    },
};

use crate::{
    domain::{
        context_menu_data::{ContextMenuData, ContextTarget},
        folder_model::FolderModel,
        request_model::RequestSavedModel,
        workspace_tree::WorkspaceTree,
    },
    enums::method::HttpMethod,
    infra::local_storage::{
        folder_repo::FolderRepository, request_repo::RequestSavedRepository,
        sqlite_repo::SqliteRepository,
    },
    ui::{
        component::context_menu_widget::ContextMenu,
        icon::icon,
        style::{method_color::get_method_color, text_input::LazyTextInput},
        theme::colors::{BG, BORDER, BTN_BG, MUTED, TEXT, VIOLET},
    },
};

pub struct SidebarState {
    pub folders: Vec<FolderModel>,
    pub root_requests: Vec<RequestSavedModel>,

    pub active_request_id: Option<i64>,
    pub current_folder_id: Option<i64>,
    pub context_menu: Option<ContextMenuData>,
}

#[derive(Debug, Clone, Copy)]
pub enum ContextAction {
    CreateRequest,
    CreateFolder,
    Duplicate,
    Delete,
}

#[derive(Debug, Clone)]
pub enum SidebarEvent {
    SelectRequest(RequestSavedModel),

    // Commands
    CreateFolder(Option<i64>),
    CreateRequest(RequestSavedModel),
    SaveCurrentRequest(RequestSavedModel),
    ReloadWorkspace,
    DeleteRequest(Option<i64>),

    // Database results
    WorkspaceRender(Result<WorkspaceTree, String>),
    RequestSaved(Result<i64, String>),

    // Context menu
    OpenContextMenu(ContextTarget, Point),
    CloseContextMenu,
    ContextAction(ContextTarget, ContextAction),
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            root_requests: Vec::new(),
            active_request_id: None,
            current_folder_id: None,
            context_menu: None,
        }
    }
    fn new_request(folder_id: Option<i64>) -> RequestSavedModel {
        RequestSavedModel {
            id: None,
            folder_id,
            name: "New Request".into(),
            method: HttpMethod::Get.to_string(),
            url: String::new(),
            headers: Vec::new(),
            params: Vec::new(),
            auth: crate::enums::auth_config::AuthConfig::None,
            body: String::new(),
            response: None,
        }
    }

    fn reload_workspace(repo: Option<&Arc<SqliteRepository>>) -> Task<SidebarEvent> {
        let Some(repo) = repo.cloned() else {
            tracing::warn!("Repository is not ready");
            return Task::none();
        };

        Task::perform(
            async move { repo.fetch_workspace_tree().await },
            SidebarEvent::WorkspaceRender,
        )
    }

    pub fn update(
        &mut self,
        event: SidebarEvent,
        repo: Option<&Arc<SqliteRepository>>,
    ) -> Task<SidebarEvent> {
        match event {
            SidebarEvent::ContextAction(target, action) => {
                self.context_menu = None;

                match (target, action) {
                    (ContextTarget::Workspace, ContextAction::CreateFolder) => {
                        self.update(SidebarEvent::CreateFolder(None), repo)
                    }

                    (ContextTarget::Folder(id), ContextAction::CreateFolder) => {
                        self.update(SidebarEvent::CreateFolder(Some(id)), repo)
                    }

                    (ContextTarget::Workspace, ContextAction::CreateRequest) => {
                        self.update(SidebarEvent::CreateRequest(Self::new_request(None)), repo)
                    }

                    (ContextTarget::Folder(id), ContextAction::CreateRequest) => self.update(
                        SidebarEvent::CreateRequest(Self::new_request(Some(id))),
                        repo,
                    ),

                    (ContextTarget::Request(id), ContextAction::Duplicate) => {
                        tracing::info!("Duplicate request {id}");
                        Task::none()
                    }

                    (ContextTarget::Request(id), ContextAction::Delete) => {
                        self.update(SidebarEvent::DeleteRequest(Some(id)), repo)
                    }

                    _ => Task::none(),
                }
            }
            SidebarEvent::OpenContextMenu(target, position) => {
                self.context_menu = Some(ContextMenuData { target, position });

                Task::none()
            }
            SidebarEvent::CloseContextMenu => {
                self.context_menu = None;
                Task::none()
            }
            // SidebarEvent::SearchCollection(_val) => Task::none(),
            SidebarEvent::SelectRequest(_data) => Task::none(),
            SidebarEvent::DeleteRequest(id) => {
                let Some(repo) = repo.cloned() else {
                    tracing::error!("Cannot delete request: repository unavailable");
                    return Task::none();
                };

                Task::perform(
                    async move {
                        repo.delete_request(id)
                            .await
                            .map(|_| id.expect("id should be exist"))
                            .map_err(|err| err.to_string())
                    },
                    SidebarEvent::RequestSaved,
                )
            }
            SidebarEvent::CreateFolder(id) => {
                if let Some(repo) = repo {
                    let repo = repo.clone();
                    Task::perform(
                        async move {
                            repo.save_folder("New Folder", id).await?;
                            repo.fetch_workspace_tree().await
                        },
                        SidebarEvent::WorkspaceRender,
                    )
                } else {
                    Task::none()
                }
            }
            SidebarEvent::CreateRequest(request) => {
                let Some(repo) = repo.cloned() else {
                    tracing::error!("Cannot create request: repository unavailable");
                    return Task::none();
                };

                Task::perform(
                    async move {
                        repo.save_request(&request)
                            .await
                            .map_err(|err| err.to_string())
                    },
                    SidebarEvent::RequestSaved,
                )
            }
            SidebarEvent::SaveCurrentRequest(request) => {
                let Some(repo) = repo.cloned() else {
                    tracing::error!("Cannot save request: repository unavailable");
                    return Task::none();
                };

                Task::perform(
                    async move {
                        let result = if request.id.is_some() {
                            repo.update_request(&request)
                                .await
                                .map(|_| request.id.expect("ID checked above"))
                        } else {
                            repo.save_request(&request).await
                        };

                        result.map_err(|err| err.to_string())
                    },
                    SidebarEvent::RequestSaved,
                )
            }
            SidebarEvent::WorkspaceRender(result) => {
                match result {
                    Ok(res) => {
                        self.folders = res.folders;
                        self.root_requests = res.requests;
                    }
                    Err(err) => tracing::error!("Failed render workspace: {err}"),
                }
                Task::none()
            }

            SidebarEvent::RequestSaved(result) => match result {
                Ok(id) => {
                    tracing::debug!("Request saved successfully: {id}");
                    Self::reload_workspace(repo)
                }

                Err(err) => {
                    tracing::error!("Failed to save request: {err}");
                    Task::none()
                }
            },

            SidebarEvent::ReloadWorkspace => Self::reload_workspace(repo),
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, SidebarEvent> {
        let header = row![
            column![
                text("Collections").size(15).color(TEXT),
                text("Your API workspaces").size(10).color(MUTED),
            ]
            .spacing(3),
            horizontal(),
            button(icon("fa-plus").size(17))
                .style(|_theme, _style| button::Style {
                    background: Background::Color(BTN_BG).into(),
                    border: iced::Border {
                        color: BORDER,
                        width: 1.0,
                        radius: 1.0.into(),
                    },
                    text_color: VIOLET,
                    ..Default::default()
                })
                .padding([3, 10]),
        ]
        .align_y(Alignment::Center);

        let search_bar = text_input("Search collections...", "")
            .padding([9, 10])
            .size(12)
            .search_style();

        let mut workspace_tree = column![].spacing(6);
        for folder in &self.folders {
            workspace_tree = workspace_tree.push(Self::render_folder(folder, 0, self.context_menu));
        }

        for req in &self.root_requests {
            workspace_tree =
                workspace_tree.push(Self::render_request_item(req, 0, self.context_menu));
        }

        let sidebar_content = container(
            column![header, search_bar, Space::new().height(5), workspace_tree,]
                .spacing(14)
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fixed(232.0))
        .height(Length::Fill)
        .padding(15)
        .style(|_| container::Style {
            background: Some(BG.into()),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        let target = ContextTarget::Workspace;

        let position = self
            .context_menu
            .and_then(|menu| (menu.target == target).then_some(menu.position));

        ContextMenu::new(
            sidebar_content,
            Self::context_menu(target),
            position,
            move |point| SidebarEvent::OpenContextMenu(target, point),
            SidebarEvent::CloseContextMenu,
        )
        .into()
    }

    fn render_folder<'a>(
        folder: &'a FolderModel,
        depth: usize,
        active_menu: Option<ContextMenuData>,
    ) -> Element<'a, SidebarEvent> {
        let indent = (depth * 12) as f32;

        let folder_header = row![
            Space::new().width(indent),
            icon("fa-folder").size(13).color(VIOLET),
            text(&folder.name).size(13).color(TEXT),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let folder_header = Self::with_context_menu(
            folder_header,
            folder.id.map(ContextTarget::Folder),
            active_menu,
        );

        let mut children = column![folder_header].spacing(4);

        for subfolder in &folder.subfolders {
            children = children.push(Self::render_folder(subfolder, depth + 1, active_menu));
        }

        for req in &folder.requests {
            children = children.push(Self::render_request_item(req, depth + 1, active_menu));
        }

        children.into()
    }

    fn render_request_item<'a>(
        req: &'a RequestSavedModel,
        depth: usize,
        active_menu: Option<ContextMenuData>,
    ) -> Element<'a, SidebarEvent> {
        let indent = (depth * 12) as f32;
        let method = req.method.parse().unwrap_or(HttpMethod::Get);
        let method_color = get_method_color(&method);

        let item_content = row![
            Space::new().width(indent),
            text(&req.method).size(9).color(method_color),
            text(&req.name).size(12).color(TEXT),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let request_content = Self::with_context_menu(
            item_content,
            req.id.map(ContextTarget::Request),
            active_menu,
        );

        container(mouse_area(request_content).on_press(SidebarEvent::SelectRequest(req.clone())))
            .width(Length::Fill)
            .padding(5.0)
            .into()
    }

    fn context_menu(target: ContextTarget) -> Element<'static, SidebarEvent> {
        use crate::ui::component::menu::{MenuItem, menu};

        let actions: &[(&str, ContextAction)] = match target {
            ContextTarget::Workspace | ContextTarget::Folder(_) => &[
                ("Create Request", ContextAction::CreateRequest),
                ("Create Folder", ContextAction::CreateFolder),
            ],
            ContextTarget::Request(_) => &[
                ("Duplicate", ContextAction::Duplicate),
                ("Delete", ContextAction::Delete),
            ],
        };

        let items = actions
            .iter()
            .map(|(label, action)| {
                MenuItem::new(*label, SidebarEvent::ContextAction(target, *action))
            })
            .collect();

        menu(items)
    }

    fn with_context_menu<'a>(
        content: impl Into<Element<'a, SidebarEvent>>,
        target: Option<ContextTarget>,
        active_menu: Option<ContextMenuData>,
    ) -> Element<'a, SidebarEvent> {
        let content = content.into();

        let Some(target) = target else {
            return content;
        };

        let position =
            active_menu.and_then(|menu| (menu.target == target).then_some(menu.position));

        ContextMenu::new(
            content,
            Self::context_menu(target),
            position,
            move |point| SidebarEvent::OpenContextMenu(target, point),
            SidebarEvent::CloseContextMenu,
        )
        .into()
    }
}
