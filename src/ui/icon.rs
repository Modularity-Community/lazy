use nerd_fonts::NerdFonts;
use std::sync::LazyLock;

static NERD_FONTS: LazyLock<NerdFonts> = LazyLock::new(|| NerdFonts {
    nf: NerdFonts::load(),
});

pub fn get_char(name: &str) -> Option<char> {
    NERD_FONTS.get(name)
}

pub fn icon<'a>(name: &str) -> iced::widget::Text<'a> {
    let c = get_char(name).unwrap_or(' ');
    iced::widget::text(c.to_string())
}
