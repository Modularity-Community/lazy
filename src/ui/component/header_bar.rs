use crate::{
    enums::method::HttpMethod,
    ui::theme::colors::{BG, BORDER, CYAN, MUTED, SURFACE, TEXT, VIOLET},
};
use iced::{
    Alignment, Element, Length, Padding, Task, color,
    widget::{
        button, column, container, pick_list, responsive, row, space::horizontal, text, text_input,
    },
};

pub struct HeaderBarState {
    pub url: String,
    pub method: HttpMethod,
    pub is_sending: bool,
}

#[derive(Debug, Clone)]
pub enum HeaderBarMessage {
    UrlEdited(String),
    SendReqeust,
    FinishedRequest,
    MethodSelected(HttpMethod),
}

impl HeaderBarState {
    pub fn new() -> Self {
        Self {
            url: String::from("https://jsonplaceholder.typicode.com/posts"),
            method: HttpMethod::Get,
            is_sending: false,
        }
    }

    pub fn update(&mut self, message: HeaderBarMessage) -> Task<HeaderBarMessage> {
        match message {
            HeaderBarMessage::UrlEdited(url) => self.url = url,
            HeaderBarMessage::FinishedRequest => self.is_sending = false,
            HeaderBarMessage::SendReqeust => self.is_sending = true,
            HeaderBarMessage::MethodSelected(method) => self.method = method,
        }
        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, HeaderBarMessage> {
        let breadcrumb = row![
            text("WORKSPACE").size(10).color(MUTED),
            text("/ ").size(11).color(MUTED),
            text("Auth API").size(12).color(VIOLET),
            horizontal(),
            text("REST CLIENT").size(10).color(MUTED),
        ]
        .align_y(Alignment::Center);

        let request_controls: Element<'a, HeaderBarMessage> = responsive(move |size| {
            let method_picker = pick_list(
                &HttpMethod::ALL[..],
                Some(&self.method),
                HeaderBarMessage::MethodSelected,
            )
            .width(Length::Fixed(104.0))
            .padding([10, 9]);

            let url_input = text_input("https://api.example.com/endpoint", &self.url)
                .on_input(HeaderBarMessage::UrlEdited)
                .padding([11, 12])
                .size(14)
                .width(Length::Fill)
                .style(|_theme, _status| iced::widget::text_input::Style {
                    background: SURFACE.into(),
                    border: iced::Border {
                        color: BORDER,
                        width: 1.0,
                        radius: 9.0.into(),
                    },
                    icon: MUTED,
                    placeholder: MUTED,
                    value: TEXT,
                    selection: VIOLET,
                });

            let send_label = if self.is_sending {
                "Sending…"
            } else {
                "Send"
            };
            let send_btn = button(text(send_label).size(13).color(BG))
                .on_press_maybe((!self.is_sending).then_some(HeaderBarMessage::SendReqeust))
                .padding([11, 18])
                .style(|_theme, status| {
                    let background = match status {
                        iced::widget::button::Status::Hovered => color!(0x9AF0F0),
                        _ => CYAN,
                    };
                    iced::widget::button::Style {
                        background: Some(background.into()),
                        text_color: BG,
                        border: iced::Border {
                            radius: 9.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });

            if size.width < 520.0 {
                column![
                    row![method_picker].width(Length::Fill),
                    url_input,
                    row![horizontal(), send_btn],
                ]
                .spacing(9)
                .width(Length::Fill)
                .into()
            } else {
                row![method_picker, url_input, send_btn]
                    .spacing(9)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .into()
            }
        })
        .into();

        container(
            column![
                container(breadcrumb).padding(Padding {
                    top: 2.0,
                    bottom: 2.0,
                    left: 10.0,
                    right: 2.0
                }),
                request_controls,
            ]
            .spacing(2)
            .padding([14, 16]),
        )
        .width(Length::Fill)
        .style(|_| container::background(SURFACE))
        .into()
    }
}
