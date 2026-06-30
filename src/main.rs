use iced::widget::{button, column, container, pick_list, row, rule, text, text_input, Space};
use iced::{Alignment, Element, Length, Task, Theme};
use std::process::Child;

// ── Codec ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Codec {
    H264,
    H265,
    VP8,
    VP9,
    AV1,
}

impl std::fmt::Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Codec::H264 => write!(f, "H.264"),
            Codec::H265 => write!(f, "H.265 / HEVC"),
            Codec::VP8 => write!(f, "VP8"),
            Codec::VP9 => write!(f, "VP9"),
            Codec::AV1 => write!(f, "AV1"),
        }
    }
}

impl Codec {
    const ALL: &'static [Codec] = &[Codec::H264, Codec::H265, Codec::VP8, Codec::VP9, Codec::AV1];

    fn wf_arg(&self) -> &'static str {
        match self {
            Codec::H264 => "libx264",
            Codec::H265 => "libx265",
            Codec::VP8 => "libvpx",
            Codec::VP9 => "libvpx-vp9",
            Codec::AV1 => "libaom-av1",
        }
    }
}

// ── Format ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Format {
    MP4,
    MKV,
    WebM,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::MP4 => write!(f, "MP4  (.mp4)"),
            Format::MKV => write!(f, "MKV  (.mkv)"),
            Format::WebM => write!(f, "WebM (.webm)"),
        }
    }
}

impl Format {
    const ALL: &'static [Format] = &[Format::MP4, Format::MKV, Format::WebM];

    fn ext(&self) -> &'static str {
        match self {
            Format::MP4 => "mp4",
            Format::MKV => "mkv",
            Format::WebM => "webm",
        }
    }
}

// ── App state ────────────────────────────────────────────────────────────────

const FPS_OPTIONS: &[u32] = &[15, 24, 30, 60, 120];

struct App {
    fps: u32,
    output_dir: String,
    filename: String,
    codec: Codec,
    format: Format,
    region: Option<String>,
    is_recording: bool,
    slurp_installed: bool,
    wf_recorder_installed: bool,
    recording_process: Option<Child>,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        // Ensure ~/Videos exists
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
            slurp_installed: cmd_exists("slurp"),
            wf_recorder_installed: cmd_exists("wf-recorder"),
            recording_process: None,
            status: "Ready".to_string(),
        }
    }
}

fn cmd_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Message {
    FpsSelected(u32),
    OutputDirChanged(String),
    FilenameChanged(String),
    BrowseDir,
    DirSelected(Option<String>),
    CodecSelected(Codec),
    FormatSelected(Format),
    SelectRegion,
    RegionSelected(Result<String, String>),
    ToggleRecording,
    InstallSlurp,
    SlurpInstalled(Result<(), String>),
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("wf-recorder GUI")
        .theme(app_theme)
        .window_size((640.0, 400.0))
        .run()
}

fn app_theme(_state: &App) -> Theme {
    Theme::Dark
}

// ── Update ───────────────────────────────────────────────────────────────────

fn update(app: &mut App, message: Message) -> Task<Message> {
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
            // Auto-switch to WebM for VP8/VP9
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
        Message::ToggleRecording => {
            if app.is_recording {
                // Stop: send SIGINT and wait in background thread
                if let Some(mut child) = app.recording_process.take() {
                    let pid = child.id();
                    std::thread::spawn(move || {
                        let _ = std::process::Command::new("kill")
                            .args(["-INT", &pid.to_string()])
                            .status();
                        let _ = child.wait();
                    });
                }
                app.is_recording = false;
                app.status = "Recording stopped".to_string();
            } else {
                let path = format!(
                    "{}/{}.{}",
                    app.output_dir,
                    app.filename,
                    app.format.ext()
                );
                let mut cmd = std::process::Command::new("wf-recorder");
                cmd.arg("-f").arg(&path);
                cmd.arg("-r").arg(app.fps.to_string());
                cmd.arg("-c").arg(app.codec.wf_arg());
                if let Some(region) = &app.region {
                    cmd.arg("-g").arg(region);
                }
                match cmd.spawn() {
                    Ok(child) => {
                        app.recording_process = Some(child);
                        app.is_recording = true;
                        app.status = format!("Recording → {}", path);
                    }
                    Err(e) => app.status = format!("Failed to start wf-recorder: {}", e),
                }
            }
            Task::none()
        }
        Message::InstallSlurp => Task::perform(
            async {
                tokio::process::Command::new("pkexec")
                    .args(["apt", "install", "-y", "slurp"])
                    .status()
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|s| {
                        if s.success() {
                            Ok(())
                        } else {
                            Err("apt install returned non-zero".to_string())
                        }
                    })
            },
            Message::SlurpInstalled,
        ),
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

// ── View ─────────────────────────────────────────────────────────────────────

fn view(app: &App) -> Element<'_, Message> {
    // ── wf-recorder missing warning ──
    let wf_warning: Option<Element<Message>> = if !app.wf_recorder_installed {
        Some(
            text("⚠  wf-recorder not installed: sudo apt install wf-recorder")
                .size(13)
                .into(),
        )
    } else {
        None
    };

    // ── Output path ──
    let path_row = row![
        text("Save to:").width(90),
        text_input("/home/…/Videos", &app.output_dir)
            .on_input(Message::OutputDirChanged)
            .width(Length::Fill),
        button("Browse").on_press(Message::BrowseDir),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let filename_row = row![
        text("Filename:").width(90),
        text_input("recording", &app.filename)
            .on_input(Message::FilenameChanged)
            .width(Length::Fill),
        text(format!(".{}", app.format.ext())).width(55),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // ── Settings ──
    let settings_row = row![
        text("FPS:"),
        pick_list(FPS_OPTIONS, Some(app.fps), Message::FpsSelected).width(75),
        Space::new().width(12),
        text("Codec:"),
        pick_list(Codec::ALL, Some(app.codec.clone()), Message::CodecSelected).width(150),
        Space::new().width(12),
        text("Format:"),
        pick_list(Format::ALL, Some(app.format.clone()), Message::FormatSelected).width(130),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // ── Region ──
    let region_row: Element<Message> = if app.slurp_installed {
        let btn_label = if app.region.is_some() {
            "↺ Re-select Region"
        } else {
            "⬚ Select Region"
        };
        row![
            button(btn_label).on_press(Message::SelectRegion),
            Space::new().width(8),
            text(
                app.region
                    .as_deref()
                    .unwrap_or("Full screen (no region selected)")
            )
            .size(13),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            text("⚠  slurp not found — region selection unavailable").size(13),
            Space::new().width(12),
            button("Install slurp").on_press(Message::InstallSlurp),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    // ── Record button ──
    let record_btn = if app.is_recording {
        button("■  Stop Recording").on_press(Message::ToggleRecording)
    } else {
        button("⏺  Start Recording").on_press(Message::ToggleRecording)
    };

    // ── Layout ──
    let mut layout = column![
        text("wf-recorder GUI").size(22),
        Space::new().height(6),
        path_row,
        filename_row,
        rule::horizontal(1),
        settings_row,
        rule::horizontal(1),
        region_row,
        Space::new().height(8),
        record_btn,
        Space::new().height(4),
        text(&app.status).size(13),
    ]
    .spacing(10)
    .padding(24);

    if let Some(warn) = wf_warning {
        layout = layout.push(warn);
    }

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
