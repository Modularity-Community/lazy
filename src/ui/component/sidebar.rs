use iced::{
    Element, Length, Task, color,
    widget::{Space, column, container, horizontal_space, row, text, text_input},
};

pub struct SidebarState {}

#[derive(Debug, Clone)]
pub enum SidebarMessage {}

impl SidebarState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, _: SidebarMessage) -> Task<SidebarMessage> {
        Task::none()
    }
    pub fn view<'a>(&'a self) -> Element<'a, SidebarMessage> {
        let header = row![
            text("Collections").size(16),
            horizontal_space(),
            text("+").size(16)
        ]
        .align_y(iced::Alignment::Center);

        let search_bar = text_input("Filter collections...", "").padding(8);

        //TODO: Integration with sqlite
        let folder_tree = column![
            text("Auth API").size(14),
            row![
                Space::with_width(20),
                text("POST - Login").size(13).color(color!(0xFFA500))
            ],
            row![
                Space::with_width(20),
                text("GET - Profile").size(13).color(color!(0x4169E1))
            ],
            row![
                Space::with_width(20),
                text("POST - Refresh Token")
                    .size(13)
                    .color(color!(0xFFA500))
            ],
            Space::with_height(10),
            text("User Management").size(14),
        ]
        .spacing(8);

        let footer =
            column![text("Trash").size(13), text("Collection Settings").size(13)].spacing(10);

        container(
            column![header, search_bar, folder_tree, horizontal_space(), footer]
                .spacing(15)
                .padding(15),
        )
        .width(Length::Fixed(250.0))
        .height(Length::Fill)
        .style(|_theme| container::background(color!(0x16161E)))
        .into()
    }
}
