use iced::{
    Color, Element, Font, Length,
    widget::{Column, Container, Text, container},
};
use iced_code_editor::{CodeEditor, Message as EditorMsg, theme};
use serde_json::Value;

use crate::ui::theme::colors::EDITOR_BG;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

pub struct JsonEditor {
    pub editor: CodeEditor,
    pub diagnostics: Vec<Diagnostic>,
    pub last_validated: String,
}

impl JsonEditor {
    pub fn new(content: &str) -> Self {
        let mut editor = CodeEditor::new(content, "json");
        let mut style = theme::from_iced_theme(&iced::Theme::SolarizedDark);
        style.background = EDITOR_BG;
        style.gutter_background = EDITOR_BG;
        editor.set_theme(style);
        editor.set_wrap_enabled(true);
        editor.set_font(Font::with_name("Fira Code"));
        let mut s = Self {
            editor,
            diagnostics: vec![],
            last_validated: content.to_string(),
        };
        s.validate();
        s
    }

    pub fn validate(&mut self) {
        let content = self.editor.content();
        if content == self.last_validated {
            return;
        }
        self.last_validated = content.clone();
        self.diagnostics = match serde_json::from_str::<Value>(&content) {
            Ok(_) => vec![],
            Err(e) => {
                let line = e.line();
                let col = e.column();
                vec![Diagnostic {
                    line: line as u32 - 1,
                    col: col as u32,
                    message: e.to_string(),
                }]
            }
        };
    }

    pub fn format(&mut self) -> iced::Task<EditorMsg> {
        let content = self.editor.content();
        match serde_json::from_str::<Value>(&content) {
            Ok(v) => {
                let pretty = serde_json::to_string_pretty(&v).unwrap_or_default();
                if pretty != content {
                    let task = self.editor.reset(&pretty);
                    self.validate();
                    task
                } else {
                    iced::Task::none()
                }
            }
            Err(_) => iced::Task::none(),
        }
    }

    pub fn set_content(&mut self, content: &str) -> iced::Task<EditorMsg> {
        let task = self.editor.reset(content);
        self.validate();
        task
    }

    pub fn view<'a>(&'a self) -> Element<'a, EditorMsg> {
        let status_bar = if self.diagnostics.is_empty() {
            container(
                Text::new("✓ Valid JSON")
                    .size(11)
                    .color(Color::from([0.3, 0.7, 0.3])),
            )
            .padding(4)
            .width(Length::Fill)
        } else {
            let diag = self
                .diagnostics
                .first()
                .map(|d| {
                    format!(
                        "Line:{} Column:{} Message: ({})",
                        d.line + 1,
                        d.col + 1,
                        d.message
                    )
                })
                .unwrap_or_default();
            Container::new(Text::new(diag).size(11).color(Color::from([0.9, 0.3, 0.3])))
                .padding(4)
                .width(Length::Fill)
        };

        Column::new()
            .push(self.editor.view())
            .push(status_bar)
            .spacing(0)
            .height(Length::Fill)
            .into()
    }
}
