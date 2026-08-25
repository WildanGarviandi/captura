//! State transitions driven by [`crate::message::Message`].
//!
//! `update` is the single entry point. Each arm handles one message and
//! returns the [`Task`] to run after the state change.

use crate::codec::Codec;
use crate::deps;
use crate::format::Format;
use crate::message::Message;
use crate::model::App;
use crate::recorder;
use iced::Task;
use notify_rust::Notification;
use std::sync::atomic::Ordering;

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::FpsSelected(fps) => {
            app.fps = fps;
            Task::none()
        }
        Message::OutputDirChanged(dir) => {
            app.output_dir = dir;
            Task::none()
        }
        Message::FilenameChanged(name) => {
            app.filename = name;
            Task::none()
        }
        Message::BrowseDir => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .set_title("Choose output directory")
                    .pick_folder()
                    .await
                    .map(|h| h.path().to_string_lossy().to_string())
            },
            Message::DirSelected,
        ),
        Message::DirSelected(path) => {
            if let Some(p) = path {
                app.output_dir = p;
            }
            Task::none()
        }
        Message::CodecSelected(codec) => {
            if matches!(codec, Codec::VP8 | Codec::VP9) {
                app.format = Format::WebM;
            } else if app.format == Format::WebM {
                app.format = Format::MKV;
            }
            app.codec = codec;
            Task::none()
        }
        Message::FormatSelected(fmt) => {
            app.format = fmt;
            Task::none()
        }
        Message::SelectRegion => Task::perform(
            async {
                tokio::process::Command::new("slurp")
                    .output()
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|out| {
                        if out.status.success() {
                            String::from_utf8(out.stdout)
                                .map(|s| s.trim().to_string())
                                .map_err(|e| e.to_string())
                        } else {
                            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
                        }
                    })
            },
            Message::RegionSelected,
        ),
        Message::RegionSelected(result) => {
            match result {
                Ok(r) if !r.is_empty() => {
                    app.status = format!("Region: {}", r);
                    app.region = Some(r);
                }
                Ok(_) => app.status = "Region selection cancelled".to_string(),
                Err(e) => app.status = format!("slurp error: {}", e),
            }
            Task::none()
        }
        Message::Tick => {
            // Check if the notification thread set the stop flag.
            if let Some(flag) = &app.notification_stop_flag {
                if flag.load(Ordering::Relaxed) {
                    // Reset the flag
                    flag.store(false, Ordering::Relaxed);
                    // Stop the recording
                    if app.is_recording {
                        stop_recording(app);
                        app.status = "Recording stopped from notification".to_string();
                    }
                }
            }
            Task::none()
        }
        Message::ToggleRecording => toggle_recording(app),
        Message::FullScreen => {
            // Clear region for full screen recording and reuse the toggle logic
            app.region = None;
            toggle_recording(app)
        }
        Message::InstallSlurp => Task::perform(deps::install_slurp(), Message::SlurpInstalled),
        Message::SlurpInstalled(result) => {
            match result {
                Ok(()) => {
                    app.slurp_installed = true;
                    app.status = "slurp installed successfully!".to_string();
                }
                Err(e) => app.status = format!("Install failed: {}", e),
            }
            Task::none()
        }
    }
}

/// Start or stop a recording, mirroring the `ToggleRecording` button behavior.
fn toggle_recording(app: &mut App) -> Task<Message> {
    if app.is_recording {
        stop_recording(app);
        app.status = "Recording stopped".to_string();
        let _ = Notification::new()
            .summary("Captura")
            .body("Recording stopped.")
            .show();
    } else {
        let started = recorder::start(
            &app.output_dir,
            &app.filename,
            &app.format,
            app.fps,
            &app.codec,
            &app.region,
        );
        match started {
            Ok(recorder::RecordingStarted { process, stop_flag }) => {
                app.recording_process = Some(process);
                app.is_recording = true;
                app.status = format!(
                    "Recording → {}",
                    recorder::output_path(&app.output_dir, &app.filename, &app.format)
                );
                app.notification_stop_flag = Some(stop_flag);
            }
            Err(e) => app.status = e,
        }
    }
    Task::none()
}

/// Kill the recording process and clean up state.
fn stop_recording(app: &mut App) {
    if let Some(child) = app.recording_process.take() {
        recorder::stop_process(child);
    }
    app.is_recording = false;
    app.notification_stop_flag = None;
}
