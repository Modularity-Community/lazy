pub mod colors {
    use iced::{Color, color};

    pub const BG: Color = Color::from_rgb(
        0x11 as f32 / 255.0,
        0x13 as f32 / 255.0,
        0x1D as f32 / 255.0,
    ); // #11131D

    pub const WORKSPACE: Color = Color::from_rgb(
        0x19 as f32 / 255.0,
        0x1C as f32 / 255.0,
        0x29 as f32 / 255.0,
    ); // #191C29

    pub const SURFACE: iced::Color = color!(0x1B1E2D);
    pub const TEXT: iced::Color = color!(0xF2F3FC);
    pub const MUTED: iced::Color = color!(0xA5ADC6);
    pub const CYAN: iced::Color = color!(0x78E6E8);
    pub const VIOLET: iced::Color = color!(0xC5B7FF);
    pub const BORDER: iced::Color = color!(0x30364B);

    pub const EDITOR_BG: iced::Color = color!(0x121520);

    pub const ORANGE: iced::Color = color!(0xFFB86B);
    pub const GREEN: iced::Color = color!(0x72E0A3);
    pub const RED: iced::Color = color!(0xFF8295);
    pub const PRIMARY: Color = Color::from_rgb(0.2, 0.4, 0.8);
    pub const TEXT_PRIMARY: Color = Color::WHITE;
    pub const TEXT_MUTED: Color = Color::from_rgb(0.7, 0.7, 0.7);
}
