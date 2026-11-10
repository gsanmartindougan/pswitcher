use iced::{Color, Theme, theme::Palette};

pub const CANVAS: Color = Color::from_rgb8(247, 245, 240);
pub const TEXT: Color = Color::from_rgb8(40, 35, 31);
pub const RED_VIOLET: Color = Color::from_rgb8(199, 21, 133);
pub const RED_VIOLET_HOVER: Color = Color::from_rgb8(161, 20, 110);
pub const RED_VIOLET_PRESSED: Color = Color::from_rgb8(133, 14, 89);
pub const ISABELLA: Color = Color::from_rgb8(195, 165, 92);
pub const SUCCESS: Color = Color::from_rgb8(51, 99, 68);
pub const WARNING: Color = Color::from_rgb8(111, 85, 26);
pub const ERROR: Color = Color::from_rgb8(157, 46, 40);

pub fn theme() -> Theme {
    Theme::custom(
        "Pswitcher",
        Palette {
            background: CANVAS,
            text: TEXT,
            primary: RED_VIOLET,
            success: SUCCESS,
            warning: WARNING,
            danger: ERROR,
        },
    )
}

pub fn primary_button(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let mut style = iced::widget::button::primary(theme, status);
    style.background = Some(
        match status {
            iced::widget::button::Status::Hovered => RED_VIOLET_HOVER,
            iced::widget::button::Status::Pressed => RED_VIOLET_PRESSED,
            _ => RED_VIOLET,
        }
        .into(),
    );
    style
}
