use crate::enums::method::HttpMethod;

use iced::{
    Element, Length, Padding, Task,
    widget::{button, pick_list, row, text, text_input},
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
    MethodSelected(crate::enums::method::HttpMethod),
}

impl HeaderBarState {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            method: HttpMethod::Get,
            is_sending: false,
        }
    }
    pub fn update(&mut self, message: HeaderBarMessage) -> Task<HeaderBarMessage> {
        match message {
            HeaderBarMessage::UrlEdited(url) => {
                self.url = url;
                Task::none()
            }
            HeaderBarMessage::FinishedRequest => {
                self.is_sending = false;

                Task::none()
            }
            HeaderBarMessage::SendReqeust => {
                self.is_sending = true;
                Task::none()
                // let url = self.url.clone();
                // self.status_text = String::from("Loading...");
                // self.time_text = String::from("-");
                // let body_text = self.request_body.text();
                //
                // let client = Client::builder()
                //     .danger_accept_invalid_hostnames(true)
                //     .danger_accept_invalid_certs(true)
                //     .build()
                //     .unwrap();
                //
                // Task::perform(
                //     async move {
                //         let start_time = std::time::Instant::now();
                //         let request = client.request(method, &url);
                //
                //         let request = if !body_text.is_empty() {
                //             request
                //                 .header("content-type", "application/json; charset=UTF-8")
                //                 .body(body_text)
                //         } else {
                //             request
                //         };
                //
                //         match request.send().await {
                //             Ok(res) => {
                //                 let status = res.status().to_string();
                //                 let time = format!("{}ms", start_time.elapsed().as_millis());
                //                 match res.text().await {
                //                     Ok(text) => Ok((text, status, time)),
                //                     Err(e) => Err(e.to_string()),
                //                 }
                //             }
                //             Err(e) => Err(e.to_string()),
                //         }
                //     },
                //     Message::ResponseReceived,
                // )
            }
            HeaderBarMessage::MethodSelected(method) => {
                self.method = method;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<HeaderBarMessage> {
        //TODO: Workspace
        // let top_bar = row![
        //     text("Auth API > Login").size(12),
        //     horizontal_space(),
        //     text("2 mins ago").size(12)
        // ]
        // .padding([10, 15]);

        let method_picker = pick_list(
            &HttpMethod::ALL[..],
            Some(&self.method),
            HeaderBarMessage::MethodSelected,
        )
        .width(Length::Fixed(90.0));

        let url_input = text_input("Enter URL...", &self.url)
            .on_input(HeaderBarMessage::UrlEdited)
            .padding(8);

        let send_btn = button(if self.is_sending {
            text("Sending...")
        } else {
            text("Send")
        })
        .on_press_maybe(if self.is_sending {
            None
        } else {
            Some(HeaderBarMessage::SendReqeust)
        })
        .padding([8, 20]);

        row![method_picker, url_input, send_btn]
            .spacing(10)
            .padding(Padding {
                top: 0.0,
                right: 15.0,
                bottom: 10.0,
                left: 15.0,
            })
            .align_y(iced::Alignment::Center)
            .into()
    }
}
