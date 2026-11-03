use crate::{
    enums::method::HttpMethod,
    ui::theme::colors::{CYAN, GREEN, ORANGE, RED},
};

pub fn get_method_color(method: &HttpMethod) -> iced::Color {
    match method {
        HttpMethod::Get => CYAN,
        HttpMethod::Post => GREEN,
        HttpMethod::Put | HttpMethod::Patch => ORANGE,
        HttpMethod::Delete => RED,
        // _ => VIOLET,
    }
}
