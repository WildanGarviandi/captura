//! `wf-recorder` GUI — entry point.
//!
//! All non-trivial logic lives in dedicated modules:
//!
//! - [`codec`], [`format`] — enumerations of supported codecs and containers.
//! - [`model`] — the `App` state and its `Default` implementation.
//! - [`message`] — events that drive the [`update`] state machine.
//! - [`update`] — pure state transitions.
//! - [`view`] — UI rendering.
//! - [`theme`] — application theme selection.
//! - [`icons`], [`styles`] — canvas-drawn icons and button styles used by [`view`].
//! - [`recorder`] — `wf-recorder` process orchestration and notification hook.
//! - [`deps`] — system dependency detection and installation.

mod codec;
mod deps;
mod format;
mod icons;
mod message;
mod model;
mod recorder;
mod styles;
mod theme;
mod update;
mod view;

use crate::message::Message;
use crate::model::App;

fn main() -> iced::Result {
    iced::application(App::default, update::update, view::view)
        .title("wf-recorder GUI")
        .theme(theme::app_theme)
        .window_size((640.0, 400.0))
        .subscription(|_| {
            // Poll the notification stop flag twice a second; the worker
            // thread flips it when the user clicks "Stop".
            iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick)
        })
        .run()
}
