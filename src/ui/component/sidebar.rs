use std::sync::Arc;

use iced::{
    Alignment, Background, Element, Length, Point, Task,
    widget::{
        self, Space, button, column, container, mouse_area, row, space::horizontal, text,
        text_input,
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
        component::{context_menu_widget::ContextMenu, rename_field::RenameField},
        icon::icon,
        style::{method_color::get_method_color, text_input::LazyTextInput},
        theme::colors::{BG, BORDER, BTN_BG, MUTED, TEXT, VIOLET},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum ContextAction {
    CreateRequest,
    CreateFolder,
    Duplicate,
    Delete,
    Rename,
    GenerateCurl,
}

#[derive(Debug, Clone)]
pub struct RenameState {
    pub target: ContextTarget,
    pub original_name: String,
    pub draft: String,
}

pub struct SidebarState {
    pub folders: Vec<FolderModel>,
    pub root_requests: Vec<RequestSavedModel>,

    pub active_request_id: Option<i64>,
    pub current_folder_id: Option<i64>,

    pub context_menu: Option<ContextMenuData>,

    pub renaming: Option<RenameState>,
    pub search_query: String,
    pub search_results: Vec<crate::infra::local_storage::request_repo::RequestSearchResult>,
    pub search_revision: u64,
    pub status: Option<String>,
    pub rename_saving: bool,
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
    DeleteFolder(Option<i64>),

    // Database results
    WorkspaceRender(Result<WorkspaceTree, String>),
    RequestSaved(Result<i64, String>),

    // Context menu
    OpenContextMenu(ContextTarget, Point),
    CloseContextMenu,
    ContextAction(ContextTarget, ContextAction),

    BeginRename(ContextTarget),
    RenameChanged(String),
    CommitRename,
    CancelRename,
    RenameFinished(Result<(), String>),
    SearchChanged(String),
    SearchDebounced(u64),
    SearchFinished(
        u64,
        Result<Vec<crate::infra::local_storage::request_repo::RequestSearchResult>, String>,
    ),
    SearchSelect(i64),
    RequestLoaded(Result<RequestSavedModel, String>),
    GenerateCurlReady(Result<String, String>),
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            root_requests: Vec::new(),
            active_request_id: None,
            current_folder_id: None,
            context_menu: None,
            renaming: None,
            search_query: String::new(),
            search_results: Vec::new(),
            search_revision: 0,
            status: None,
            rename_saving: false,
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

                    (ContextTarget::Folder(id), ContextAction::Delete) => {
                        self.update(SidebarEvent::DeleteFolder(Some(id)), repo)
                    }

                    (ContextTarget::Folder(id), ContextAction::Rename) => {
                        self.update(SidebarEvent::BeginRename(ContextTarget::Folder(id)), repo)
                    }

                    (ContextTarget::Request(id), ContextAction::Rename) => {
                        self.update(SidebarEvent::BeginRename(ContextTarget::Request(id)), repo)
                    }

                    (ContextTarget::Request(id), ContextAction::GenerateCurl) => {
                        let Some(repo) = repo.cloned() else {
                            return Task::none();
                        };
                        Task::perform(
                            async move {
                                let request = repo.fetch_request_by_id(id).await?;
                                Ok(crate::utils::curl::generate_curl_for_request(&request))
                            },
                            SidebarEvent::GenerateCurlReady,
                        )
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
            SidebarEvent::DeleteFolder(id) => {
                let Some(repo) = repo.cloned() else {
                    tracing::error!("Cannot delete request: repository unavailable");
                    return Task::none();
                };
                Task::perform(
                    async move {
                        repo.delete_folder(id)
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
                        let result = if let Some(id) = request.id {
                            // Keep stored metadata, which is not represented in the editor yet.
                            // In particular, do not overwrite a renamed item with "New Request".
                            match repo.fetch_request_by_id(id).await {
                                Ok(mut existing) => {
                                    existing.method = request.method;
                                    existing.url = request.url;
                                    existing.body = request.body;
                                    existing.response = request.response;
                                    repo.update_request(&existing).await.map(|_| id)
                                }
                                Err(err) => Err(err),
                            }
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
            SidebarEvent::BeginRename(target) => {
                let Some(name) = self.target_name(target) else {
                    return Task::none();
                };

                if self.rename_saving {
                    return Task::none();
                }
                self.renaming = Some(RenameState {
                    target,
                    original_name: name.to_owned(),
                    draft: name.to_owned(),
                });

                self.context_menu = None;
                let id = widget::Id::new("sidebar-rename");
                Task::batch([
                    iced::widget::operation::focus(id.clone()),
                    iced::widget::operation::select_all(id),
                ])
            }
            SidebarEvent::RenameChanged(value) => {
                if let Some(state) = &mut self.renaming {
                    state.draft = value;
                }

                Task::none()
            }
            SidebarEvent::CancelRename => {
                if !self.rename_saving {
                    self.renaming = None;
                }
                Task::none()
            }
            SidebarEvent::CommitRename => {
                if self.rename_saving {
                    return Task::none();
                }
                let Some(state) = self.renaming.clone() else {
                    return Task::none();
                };

                let name = state.draft.trim().to_owned();
                if name.is_empty() {
                    self.status = Some("Name cannot be empty".to_owned());
                    return Task::none();
                }

                if name == state.original_name {
                    self.renaming = None;
                    return Task::none();
                }

                let Some(repo) = repo.cloned() else {
                    return Task::none();
                };

                self.rename_saving = true;
                Task::perform(
                    async move {
                        match state.target {
                            ContextTarget::Folder(id) => repo
                                .rename_folder(Some(id), &name)
                                .await
                                .map_err(|err| err.to_string()),

                            ContextTarget::Request(id) => repo
                                .rename_request(Some(id), &name)
                                .await
                                .map_err(|err| err.to_string()),

                            ContextTarget::Workspace => Ok(()),
                        }
                    },
                    SidebarEvent::RenameFinished,
                )
            }
            SidebarEvent::RenameFinished(result) => {
                self.rename_saving = false;
                match result {
                    Ok(()) => {
                        self.renaming = None;
                        Self::reload_workspace(repo)
                    }
                    Err(err) => {
                        self.status = Some(format!("Rename failed: {err}"));
                        tracing::error!("Rename failed: {err}");
                        Task::none()
                    }
                }
            }
            SidebarEvent::SearchChanged(value) => {
                self.search_query = value;
                self.search_revision = self.search_revision.wrapping_add(1);
                if self.search_query.trim().is_empty() {
                    self.search_results.clear();
                    return Task::none();
                }
                let revision = self.search_revision;
                Task::perform(
                    async move {
                        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                        revision
                    },
                    SidebarEvent::SearchDebounced,
                )
            }
            SidebarEvent::SearchDebounced(revision) => {
                if revision != self.search_revision {
                    return Task::none();
                }
                let Some(repo) = repo.cloned() else {
                    return Task::none();
                };
                let query = self.search_query.trim().to_owned();
                Task::perform(
                    async move { (revision, repo.search_requests(&query).await) },
                    |(revision, result)| SidebarEvent::SearchFinished(revision, result),
                )
            }
            SidebarEvent::SearchFinished(revision, result) => {
                if revision == self.search_revision {
                    match result {
                        Ok(items) => self.search_results = items,
                        Err(err) => self.status = Some(format!("Search failed: {err}")),
                    }
                }
                Task::none()
            }
            SidebarEvent::SearchSelect(id) => {
                let Some(repo) = repo.cloned() else {
                    return Task::none();
                };
                Task::perform(
                    async move { repo.fetch_request_by_id(id).await },
                    SidebarEvent::RequestLoaded,
                )
            }
            SidebarEvent::RequestLoaded(result) => match result {
                Ok(req) => Task::done(SidebarEvent::SelectRequest(req)),
                Err(err) => {
                    self.status = Some(err);
                    Task::none()
                }
            },
            SidebarEvent::GenerateCurlReady(result) => match result {
                Ok(command) => {
                    self.status = Some("cURL copied to clipboard".to_owned());
                    iced::clipboard::write(command)
                }
                Err(err) => {
                    self.status = Some(format!("cURL failed: {err}"));
                    Task::none()
                }
            },
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

        let search_bar = text_input("Search collections...", &self.search_query)
            .on_input(SidebarEvent::SearchChanged)
            .padding([9, 10])
            .size(12)
            .search_style();

        let mut workspace_tree = column![].spacing(6);
        if self.search_query.trim().is_empty() {
            for folder in &self.folders {
                workspace_tree =
                    workspace_tree.push(self.render_folder(folder, 0, self.context_menu));
            }
            for req in &self.root_requests {
                workspace_tree =
                    workspace_tree.push(self.render_request_item(req, 0, self.context_menu));
            }
        } else {
            for item in &self.search_results {
                let label = format!("{}  {}", item.method, item.name);
                workspace_tree = workspace_tree.push(
                    button(text(label).size(12))
                        .on_press(SidebarEvent::SearchSelect(item.id))
                        .width(Length::Fill),
                );
            }
            if self.search_results.is_empty() {
                workspace_tree =
                    workspace_tree.push(text("No matching requests").size(11).color(MUTED));
            }
        }
        let mut contents = column![header, search_bar, Space::new().height(5), workspace_tree];
        if let Some(status) = &self.status {
            contents = contents.push(text(status).size(10).color(MUTED));
        }
        let sidebar_content = container(
            contents
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
        &'a self,
        folder: &'a FolderModel,
        depth: usize,
        active_menu: Option<ContextMenuData>,
    ) -> Element<'a, SidebarEvent> {
        let indent = (depth * 12) as f32;

        let name: Element<'_, SidebarEvent> = match folder.id {
            Some(id) => self.rename_input(ContextTarget::Folder(id), &folder.name),
            None => text(&folder.name).size(13).color(TEXT).into(),
        };

        let folder_header = row![
            Space::new().width(indent),
            icon("fa-folder").size(13).color(VIOLET),
            name
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let folder_header: Element<'_, SidebarEvent> = match folder.id {
            Some(id) if !self.is_renaming(ContextTarget::Folder(id)) => mouse_area(folder_header)
                .on_double_click(SidebarEvent::BeginRename(ContextTarget::Folder(id)))
                .into(),
            _ => folder_header.into(),
        };
        let folder_header = Self::with_context_menu(
            folder_header,
            folder.id.map(ContextTarget::Folder),
            active_menu,
        );

        let mut children = column![folder_header].spacing(4);

        for subfolder in &folder.subfolders {
            children = children.push(self.render_folder(subfolder, depth + 1, active_menu));
        }

        for req in &folder.requests {
            children = children.push(self.render_request_item(req, depth + 1, active_menu));
        }

        children.into()
    }

    fn render_request_item<'a>(
        &'a self,
        req: &'a RequestSavedModel,
        depth: usize,
        active_menu: Option<ContextMenuData>,
    ) -> Element<'a, SidebarEvent> {
        let indent = (depth * 12) as f32;
        let method = req.method.parse().unwrap_or(HttpMethod::Get);
        let method_color = get_method_color(&method);

        let name: Element<'_, SidebarEvent> = match req.id {
            Some(id) => self.rename_input(ContextTarget::Request(id), &req.name),
            None => text(&req.name).size(12).into(),
        };

        let item_content = row![
            Space::new().width(indent),
            text(&req.method).size(9).color(method_color),
            name
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let editing = req
            .id
            .is_some_and(|id| self.is_renaming(ContextTarget::Request(id)));
        let item_content: Element<'_, SidebarEvent> = match req.id {
            Some(id) if !editing => mouse_area(item_content)
                .on_double_click(SidebarEvent::BeginRename(ContextTarget::Request(id)))
                .into(),
            _ => item_content.into(),
        };
        let request_content = Self::with_context_menu(
            item_content,
            req.id.map(ContextTarget::Request),
            active_menu,
        );

        let content: Element<'_, SidebarEvent> = if editing {
            request_content
        } else {
            mouse_area(request_content)
                .on_press(SidebarEvent::SelectRequest(req.clone()))
                .into()
        };
        container(content).width(Length::Fill).padding(5.0).into()
    }

    fn context_menu(target: ContextTarget) -> Element<'static, SidebarEvent> {
        use crate::ui::component::menu::{MenuItem, menu};

        let actions: &[(&str, ContextAction)] = match target {
            ContextTarget::Workspace => &[
                ("Create Request", ContextAction::CreateRequest),
                ("Create Folder", ContextAction::CreateFolder),
            ],
            ContextTarget::Folder(_) => &[
                ("Create Request", ContextAction::CreateRequest),
                ("Create Folder", ContextAction::CreateFolder),
                ("Rename", ContextAction::Rename),
                ("Delete", ContextAction::Delete),
            ],
            ContextTarget::Request(_) => &[
                ("Rename", ContextAction::Rename),
                ("Duplicate", ContextAction::Duplicate),
                ("Generate cURL", ContextAction::GenerateCurl),
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

    fn find_folder_name(folders: &[FolderModel], id: i64) -> Option<&str> {
        for folder in folders {
            if folder.id == Some(id) {
                return Some(&folder.name);
            }

            if let Some(name) = Self::find_folder_name(&folder.subfolders, id) {
                return Some(name);
            }
        }

        None
    }

    fn find_request_name(folders: &[FolderModel], id: i64) -> Option<&str> {
        for folder in folders {
            if let Some(req) = folder.requests.iter().find(|req| req.id == Some(id)) {
                return Some(&req.name);
            }

            if let Some(name) = Self::find_request_name(&folder.subfolders, id) {
                return Some(name);
            }
        }

        None
    }

    fn target_name(&self, target: ContextTarget) -> Option<&str> {
        match target {
            ContextTarget::Folder(id) => Self::find_folder_name(&self.folders, id),

            ContextTarget::Request(id) => self
                .root_requests
                .iter()
                .find(|req| req.id == Some(id))
                .map(|req| req.name.as_str())
                .or_else(|| Self::find_request_name(&self.folders, id)),

            ContextTarget::Workspace => None,
        }
    }

    fn is_renaming(&self, target: ContextTarget) -> bool {
        self.renaming
            .as_ref()
            .is_some_and(|state| state.target == target)
    }

    fn rename_input<'a>(
        &'a self,
        target: ContextTarget,
        name: &'a str,
    ) -> Element<'a, SidebarEvent> {
        if let Some(state) = &self.renaming {
            if state.target == target {
                let id = widget::Id::new("sidebar-rename");
                let input = text_input("Name", &state.draft)
                    .id(id)
                    .on_input(SidebarEvent::RenameChanged)
                    .on_submit(SidebarEvent::CommitRename)
                    .size(12)
                    .padding(4);
                return RenameField::new(
                    input,
                    SidebarEvent::CommitRename,
                    SidebarEvent::CancelRename,
                )
                .into();
            }
        }

        text(name).size(12).color(TEXT).into()
    }
}
