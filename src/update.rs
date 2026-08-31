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

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App {
            fps: 30,
            output_dir: "/tmp".to_string(),
            filename: "recording".to_string(),
            codec: Codec::H264,
            format: Format::MP4,
            region: None,
            is_recording: false,
            slurp_installed: false,
            wf_recorder_installed: false,
            recording_process: None,
            status: "Ready".to_string(),
            notification_stop_flag: None,
        }
    }

    fn apply(app: &mut App, message: Message) {
        let _ = update(app, message);
    }

    #[test]
    fn updates_recording_settings() {
        let mut app = app();

        apply(&mut app, Message::FpsSelected(60));
        apply(
            &mut app,
            Message::OutputDirChanged("/home/user/Videos".to_string()),
        );
        apply(&mut app, Message::FilenameChanged("demo".to_string()));
        apply(&mut app, Message::DirSelected(None));
        apply(
            &mut app,
            Message::DirSelected(Some("/mnt/recordings".to_string())),
        );
        apply(&mut app, Message::FormatSelected(Format::MKV));

        assert_eq!(app.fps, 60);
        assert_eq!(app.output_dir, "/mnt/recordings");
        assert_eq!(app.filename, "demo");
        assert_eq!(app.format, Format::MKV);
    }

    #[test]
    fn selects_compatible_format_for_codec() {
        let mut app = app();

        apply(&mut app, Message::CodecSelected(Codec::VP9));
        assert_eq!(app.codec, Codec::VP9);
        assert_eq!(app.format, Format::WebM);

        apply(&mut app, Message::CodecSelected(Codec::H265));
        assert_eq!(app.codec, Codec::H265);
        assert_eq!(app.format, Format::MKV);

        apply(&mut app, Message::CodecSelected(Codec::AV1));
        assert_eq!(app.codec, Codec::AV1);
        assert_eq!(app.format, Format::MKV);
    }

    #[test]
    fn records_region_selection_outcomes() {
        let mut app = app();

        apply(
            &mut app,
            Message::RegionSelected(Ok("100,200 800x600".to_string())),
        );
        assert_eq!(app.region.as_deref(), Some("100,200 800x600"));
        assert_eq!(app.status, "Region: 100,200 800x600");

        apply(&mut app, Message::RegionSelected(Ok(String::new())));
        assert_eq!(app.region.as_deref(), Some("100,200 800x600"));
        assert_eq!(app.status, "Region selection cancelled");

        apply(
            &mut app,
            Message::RegionSelected(Err("slurp unavailable".to_string())),
        );
        assert_eq!(app.status, "slurp error: slurp unavailable");
    }

    #[test]
    fn reports_slurp_installation_result() {
        let mut app = app();

        apply(&mut app, Message::SlurpInstalled(Ok(())));
        assert!(app.slurp_installed);
        assert_eq!(app.status, "slurp installed successfully!");

        apply(
            &mut app,
            Message::SlurpInstalled(Err("permission denied".to_string())),
        );
        assert!(app.slurp_installed);
        assert_eq!(app.status, "Install failed: permission denied");
    }
}
