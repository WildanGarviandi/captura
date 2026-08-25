//! Reusable button styles for the UI.

use iced::widget::button;
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Lighten or darken a color by a fixed amount, clamped to `[0, 1]`.
pub fn shade(c: Color, amount: f32) -> Color {
    Color::from_rgb(
        (c.r + amount).clamp(0.0, 1.0),
        (c.g + amount).clamp(0.0, 1.0),
        (c.b + amount).clamp(0.0, 1.0),
    )
}

/// Pill-shaped record button: red while recording, neutral dark when idle.
pub fn record_button_style(is_recording: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let base = if is_recording {
            Color::from_rgb(0.86, 0.16, 0.16) // red while actively recording
        } else {
            Color::from_rgb(0.18, 0.18, 0.20) // dark neutral when idle
        };

        let background = match status {
            button::Status::Hovered => shade(base, 0.08),
            button::Status::Pressed => shade(base, -0.08),
            button::Status::Disabled => shade(base, -0.15),
            button::Status::Active => base,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 24.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            ..button::Style::default()
        }
    }
}

/// Subtle dark rounded button used for secondary actions.
pub fn secondary_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_theme: &Theme, status: button::Status| {
        let base = Color::from_rgb(0.16, 0.16, 0.18);
        let background = match status {
            button::Status::Hovered => shade(base, 0.06),
            button::Status::Pressed => shade(base, -0.06),
            button::Status::Disabled => shade(base, -0.12),
            button::Status::Active => base,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            ..button::Style::default()
        }
    }
}

/// Amber-tinted button used to draw attention to a warning CTA.
pub fn warning_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_theme: &Theme, status: button::Status| {
        let base = Color::from_rgb(0.85, 0.55, 0.10);
        let background = match status {
            button::Status::Hovered => shade(base, 0.06),
            button::Status::Pressed => shade(base, -0.06),
            button::Status::Disabled => shade(base, -0.12),
            button::Status::Active => base,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            ..button::Style::default()
        }
    }
}
