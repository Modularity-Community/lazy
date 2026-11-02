use std::sync::Arc;

use iced::{
    Alignment, Background, Element, Length, Task,
    widget::{Space, button, column, container, row, space::horizontal, text, text_input},
};

use crate::{
    domain::{
        folder_model::FolderModel, request_model::RequestSavedModel, workspace_tree::WorkspaceTree,
    },
    infra::local_storage::{
        folder_repo::FolderRepository, request_repo::RequestSavedRepository,
        sqlite_repo::SqliteRepository,
    },
    ui::{
        icon::icon,
        style::text_input::LazyTextInput,
        theme::colors::{BG, BORDER, BTN_BG, CYAN, GREEN, MUTED, TEXT, VIOLET},
    },
};

pub struct SidebarState {
    pub folders: Vec<FolderModel>,
    pub root_requests: Vec<RequestSavedModel>,

    pub active_request_id: Option<i64>,
    pub current_folder_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum SidebarEvent {
    SearchCollection(String),
    SelectRequest(RequestSavedModel),
    CreateFolder(Option<i64>),        //parent_id
    CreateRequest(RequestSavedModel), //folder_id
    SaveCurrentRequest(RequestSavedModel),

    WorkspaceLoaded,
    WorkspaceRender(Result<WorkspaceTree, String>),
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            root_requests: Vec::new(),
            active_request_id: None,
            current_folder_id: None,
        }
    }
    pub fn update(
        &mut self,
        event: SidebarEvent,
        repo: Option<&Arc<SqliteRepository>>,
    ) -> Task<SidebarEvent> {
        match event {
            SidebarEvent::SearchCollection(val) => Task::none(),
            SidebarEvent::SelectRequest(data) => Task::none(),
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
                if let Some(repo) = repo {
                    let repo = repo.clone();
                    // let new_req = RequestSavedModel {
                    //     id: None,
                    //     folder_id: id,
                    //     name: "Untitled Request".to_string(),
                    //     method: "GET".to_string(),
                    //     url: "https://httpbin.org/get".to_string(),
                    //     headers: vec![],
                    //     params: vec![],
                    //     auth: AuthConfig::None,
                    //     body: String::new(),
                    // };

                    Task::perform(
                        async move {
                            repo.save_request(&request).await?;
                            repo.fetch_workspace_tree().await
                        },
                        SidebarEvent::WorkspaceRender,
                    )
                } else {
                    Task::none()
                }
            }
            SidebarEvent::SaveCurrentRequest(req_model) => {
                if let Some(repo) = repo {
                    let repo = repo.clone();
                    Task::perform(
                        async move {
                            if req_model.id.is_some() {
                                repo.update_request(&req_model).await?;
                                Ok(req_model.id.unwrap())
                            } else {
                                repo.save_request(&req_model).await
                            }
                        },
                        |_e| SidebarEvent::WorkspaceLoaded,
                    )
                } else {
                    tracing::error!("failed to save request ");
                    Task::none()
                }
            }
            SidebarEvent::WorkspaceLoaded => {
                if let Some(repo) = repo {
                    let repo = repo.clone();
                    Task::perform(
                        async move { repo.fetch_workspace_tree().await },
                        SidebarEvent::WorkspaceRender,
                    )
                } else {
                    tracing::warn!("Repository belum siap");
                    Task::none()
                }
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
            workspace_tree = workspace_tree.push(Self::render_folder(folder, 0));
        }

        for req in &self.root_requests {
            workspace_tree = workspace_tree.push(Self::render_request_item(req, 0));
        }
        // let collection = row![
        //     icon("fa-folder_open").size(13).color(VIOLET),
        //     text("Auth API").size(13).color(TEXT),
        //     horizontal(),
        //     text("3").size(10).color(MUTED),
        // ]
        // .spacing(8)
        // .align_y(Alignment::Center);

        // let requests = column![
        //     row![
        //         Space::new().width(15),
        //         text("POST").size(10).color(GREEN),
        //         text("Login").size(12).color(TEXT)
        //     ]
        //     .spacing(8)
        //     .align_y(Alignment::Center),
        //     row![
        //         Space::new().width(15),
        //         text("GET").size(10).color(CYAN),
        //         text("Profile").size(12).color(MUTED)
        //     ]
        //     .spacing(8)
        //     .align_y(Alignment::Center),
        //     row![
        //         Space::new().width(15),
        //         text("POST").size(10).color(GREEN),
        //         text("Refresh Token").size(12).color(MUTED)
        //     ]
        //     .spacing(8)
        //     .align_y(Alignment::Center),
        // ]
        // .spacing(13);
        //
        // let other_collection = row![
        //     icon("fa-folder").size(13).color(VIOLET),
        //     text("User Management").size(13).color(TEXT)
        // ]
        // .spacing(8)
        // .align_y(Alignment::Center);

        container(
            column![header, search_bar, Space::new().width(5), workspace_tree]
                .spacing(14)
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fixed(232.0))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(BG.into()),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn render_folder<'a>(folder: &'a FolderModel, depth: usize) -> Element<'a, SidebarEvent> {
        let indent = (depth * 12) as f32;

        let folder_header = row![
            Space::new().width(indent),
            icon("fa-folder").size(13).color(VIOLET),
            text(&folder.name).size(13).color(TEXT),
            horizontal(),
            button(text("+").size(10))
                // .on_press(SidebarEvent::CreateRequest())
                .padding([2, 5]),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let mut children = column![folder_header].spacing(4);

        for subfolder in &folder.subfolders {
            children = children.push(Self::render_folder(subfolder, depth + 1));
        }

        for req in &folder.requests {
            children = children.push(Self::render_request_item(req, depth + 1));
        }

        children.into()
    }

    fn render_request_item<'a>(
        req: &'a RequestSavedModel,
        depth: usize,
    ) -> Element<'a, SidebarEvent> {
        let indent = (depth * 12) as f32;
        let method_color = match req.method.as_str() {
            "GET" => CYAN,
            "POST" => GREEN,
            _ => VIOLET,
        };

        button(
            row![
                Space::new().width(indent),
                text(&req.method).size(9).color(method_color),
                text(&req.name).size(12).color(TEXT),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(SidebarEvent::SelectRequest(req.clone()))
        .style(|_, _| button::Style::default())
        .into()
    }
}
