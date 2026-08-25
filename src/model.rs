//! Application state and its initial/default value.

use crate::codec::Codec;
use crate::deps;
use crate::format::Format;
use std::process::Child;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub const FPS_OPTIONS: &[u32] = &[15, 24, 30, 60, 120];

pub struct App {
    pub fps: u32,
    pub output_dir: String,
    pub filename: String,
    pub codec: Codec,
    pub format: Format,
    pub region: Option<String>,
    pub is_recording: bool,
    pub slurp_installed: bool,
    pub wf_recorder_installed: bool,
    pub recording_process: Option<Child>,
    pub status: String,
    /// Shared flag: set by the notification thread when user clicks Stop.
    pub notification_stop_flag: Option<Arc<AtomicBool>>,
}

impl Default for App {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let videos = format!("{}/Videos", home);
        let _ = std::fs::create_dir_all(&videos);

        App {
            fps: 30,
            output_dir: videos,
            filename: "recording".to_string(),
            codec: Codec::H264,
            format: Format::MP4,
            region: None,
            is_recording: false,
            slurp_installed: deps::cmd_exists("slurp"),
            wf_recorder_installed: deps::cmd_exists("wf-recorder"),
            recording_process: None,
            status: "Ready".to_string(),
            notification_stop_flag: None,
        }
    }
}
