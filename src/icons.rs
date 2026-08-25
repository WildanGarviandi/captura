//! Canvas-drawn icons used inside buttons.
//!
//! Each icon implements `canvas::Program` so it can be embedded into the view
//! via `Canvas::new(...)`.

use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme, mouse};

/// Record button icon: a filled red dot when idle, a rounded white square
/// while recording.
pub struct RecordIcon {
    pub is_recording: bool,
}

impl<Message> canvas::Program<Message> for RecordIcon {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();

        if self.is_recording {
            let side = 12.0;
            let top_left = Point::new(center.x - side / 2.0, center.y - side / 2.0);
            let square = Path::rounded_rectangle(top_left, Size::new(side, side), 4.0.into());
            frame.fill(&square, Color::WHITE);
        } else {
            let circle = Path::circle(center, 7.0);
            frame.fill(&circle, Color::from_rgb(0.86, 0.16, 0.16));
        }

        vec![frame.into_geometry()]
    }
}

/// Region-selection icon: four corner brackets, optionally with a center dot
/// when a region is already selected.
pub struct RegionIcon {
    pub selected: bool,
}

impl<Message> canvas::Program<Message> for RegionIcon {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let size = bounds.width.min(bounds.height);
        let pad = size * 0.12;
        let arm = size * 0.32;

        let color = if self.selected {
            Color::from_rgb(0.30, 0.55, 0.95)
        } else {
            Color::from_rgb(0.55, 0.55, 0.58)
        };

        // (corner point, direction the two arms point in)
        let corners = [
            (Point::new(pad, pad), 1.0, 1.0),
            (Point::new(size - pad, pad), -1.0, 1.0),
            (Point::new(pad, size - pad), 1.0, -1.0),
            (Point::new(size - pad, size - pad), -1.0, -1.0),
        ];

        for (corner, hx, vy) in corners {
            let bracket = Path::new(|builder| {
                builder.move_to(Point::new(corner.x + arm * hx, corner.y));
                builder.line_to(corner);
                builder.line_to(Point::new(corner.x, corner.y + arm * vy));
            });
            frame.stroke(
                &bracket,
                Stroke {
                    width: 1.6,
                    style: canvas::stroke::Style::Solid(color),
                    ..Stroke::default()
                },
            );
        }

        if self.selected {
            frame.fill(&Path::circle(frame.center(), size * 0.06), color);
        }

        vec![frame.into_geometry()]
    }
}

/// Warning triangle icon, used next to the "Install slurp" CTA.
pub struct WarningIcon;

impl<Message> canvas::Program<Message> for WarningIcon {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let size = bounds.width.min(bounds.height);
        let pad = size * 0.08;
        let ink = Color::from_rgb(0.20, 0.14, 0.02);

        let triangle = Path::new(|builder| {
            builder.move_to(Point::new(size / 2.0, pad));
            builder.line_to(Point::new(size - pad, size - pad));
            builder.line_to(Point::new(pad, size - pad));
            builder.close();
        });
        frame.fill(&triangle, Color::from_rgb(0.95, 0.62, 0.12));

        let stem = Path::rounded_rectangle(
            Point::new(size / 2.0 - 0.8, size * 0.38),
            Size::new(1.6, size * 0.28),
            1.0.into(),
        );
        frame.fill(&stem, ink);
        frame.fill(&Path::circle(Point::new(size / 2.0, size * 0.76), 1.2), ink);

        vec![frame.into_geometry()]
    }
}
