//! Side-effect orchestration for `wf-recorder` and its notification hook.
//!
//! Keeping these behind dedicated functions keeps the [`crate::update`] logic
//! deterministic and easy to reason about.

use crate::codec::Codec;
use crate::format::Format;
use notify_rust::Notification;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The result of spawning a recording session.
pub struct RecordingStarted {
    pub process: Child,
    /// Shared flag written by the notification thread when the user requests
    /// a stop; the main thread polls it on each [`crate::message::Message::Tick`].
    pub stop_flag: Arc<AtomicBool>,
}

/// Build the `wf-recorder` command for the given settings.
fn recording_command(
    output_path: String,
    fps: u32,
    codec: &Codec,
    region: &Option<String>,
) -> std::process::Command {
    let mut cmd = std::process::Command::new("wf-recorder");
    cmd.arg("-f").arg(&output_path);
    cmd.arg("-r").arg(fps.to_string());
    cmd.arg("-c").arg(codec.wf_arg());
    if let Some(region) = region {
        cmd.arg("-g").arg(region);
    }
    cmd
}

/// Resolve the padded output file path for the current settings.
pub fn output_path(dir: &str, name: &str, format: &Format) -> String {
    format!("{}/{}.{}", dir, name, format.ext())
}

/// Spawn the `wf-recorder` process and a notification that can stop it.
pub fn start(output_dir: &str, filename: &str, format: &Format, fps: u32, codec: &Codec, region: &Option<String>) -> Result<RecordingStarted, String> {
    let path = output_path(output_dir, filename, format);
    let mut cmd = recording_command(&path, fps, codec, region);

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start wf-recorder: {}", e))?;

    // Create a shared stop flag for the notification thread.
    let stop_flag = Arc::new(AtomicBool::new(false));
    let thread_flag = stop_flag.clone();

    // Spawn notification with a "Stop" action button.
    std::thread::spawn(move || {
        let mut notification = Notification::new();
        notification
            .summary("Captura - Recording Active")
            .body("Click 'Stop' or the notification to stop recording.")
            .icon("media-record")
            .appname("Captura")
            .timeout(notify_rust::Timeout::Never);

        notification.action("stop", "Stop Recording");

        let handle = notification.show();
        if let Ok(nh) = handle {
            nh.wait_for_action(|action| {
                if action == "default" || action == "stop" {
                    // Kill wf-recorder
                    let _ = std::process::Command::new("pkill")
                        .arg("-INT")
                        .arg("wf-recorder")
                        .status();
                    // Signal the main thread to update GUI state
                    thread_flag.store(true, Ordering::Relaxed);
                }
            });
        }
    });

    Ok(RecordingStarted {
        process: child,
        stop_flag,
    })
}

/// Kill the recording process and wait for it to exit.
pub fn stop_process(mut child: Child) {
    let pid = child.id();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("kill")
            .args(["-INT", &pid.to_string()])
            .status();
        let _ = child.wait();
    });
}