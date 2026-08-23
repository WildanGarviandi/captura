use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::widget::{Space, button, column, container, pick_list, row, rule, text, text_input};
use iced::{
    Alignment, Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Shadow,
    Size, Task, Theme, Vector, mouse,
};
use notify_rust::Notification;
use std::process::Child;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
    /// Shared flag: set by the notification thread when user clicks Stop.
    notification_stop_flag: Option<Arc<AtomicBool>>,
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
            slurp_installed: cmd_exists("slurp"),
            wf_recorder_installed: cmd_exists("wf-recorder"),
            recording_process: None,
            status: "Ready".to_string(),
            notification_stop_flag: None,
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
    /// Fired every ~500ms by a subscription to check if the notification
    /// stop button was clicked.
    Tick,
}

// ── Button Style ─────────────────────────────────────────────────────────────────────

struct RecordIcon {
    is_recording: bool,
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

fn record_button_style(is_recording: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
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

fn shade(c: Color, amount: f32) -> Color {
    Color::from_rgb(
        (c.r + amount).clamp(0.0, 1.0),
        (c.g + amount).clamp(0.0, 1.0),
        (c.b + amount).clamp(0.0, 1.0),
    )
}

// --- Icons ---------------------------------------------------------------

struct RegionIcon {
    selected: bool,
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

struct WarningIcon;

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

// --- Button styles ---------------------------------------------------------

fn secondary_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
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

fn warning_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
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

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("wf-recorder GUI")
        .theme(app_theme)
        .window_size((640.0, 400.0))
        .subscription(|_| {
            // We use a simple timer subscription to poll the stop flag.
            iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick)
        })
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
        Message::ToggleRecording => {
            if app.is_recording {
                stop_recording(app);
                app.status = "Recording stopped".to_string();
                let _ = Notification::new()
                    .summary("Captura")
                    .body("Recording stopped.")
                    .show();
            } else {
                let path = format!("{}/{}.{}", app.output_dir, app.filename, app.format.ext());
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

                        // Create a shared stop flag for the notification thread.
                        let stop_flag = Arc::new(AtomicBool::new(false));
                        app.notification_stop_flag = Some(stop_flag.clone());

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
                                        stop_flag.store(true, Ordering::Relaxed);
                                    }
                                });
                            }
                        });
                    }
                    Err(e) => app.status = format!("Failed to start wf-recorder: {}", e),
                }
            }
            Task::none()
        }
        Message::InstallSlurp => Task::perform(
            async {
                let script = r#"
                if command -v pacman >/dev/null 2>&1; then
                    pacman -S --noconfirm slurp
                elif command -v dnf >/dev/null 2>&1; then
                    dnf install -y slurp
                elif command -v apt-get >/dev/null 2>&1; then
                    apt-get update && apt-get install -y slurp
                elif command -v zypper >/dev/null 2>&1; then
                    zypper install -y slurp
                elif command -v apk >/dev/null 2>&1; then
                    apk add slurp
                else
                    exit 1
                fi
                "#;

                tokio::process::Command::new("pkexec")
                    .arg("sh")
                    .arg("-c")
                    .arg(script)
                    .status()
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|s| {
                        if s.success() {
                            Ok(())
                        } else {
                            Err("Installation failed or unsupported package manager".to_string())
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

/// Kill the recording process and clean up state.
fn stop_recording(app: &mut App) {
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
    app.notification_stop_flag = None;
}

// ── View ─────────────────────────────────────────────────────────────────────

fn view(app: &App) -> Element<'_, Message> {
    let wf_warning: Option<Element<Message>> = if !app.wf_recorder_installed {
        Some(
            text("⚠  wf-recorder not installed: sudo apt install wf-recorder")
                .size(13)
                .into(),
        )
    } else {
        None
    };

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

    let settings_row = row![
        text("FPS:"),
        pick_list(FPS_OPTIONS, Some(app.fps), Message::FpsSelected).width(75),
        Space::new().width(12),
        text("Codec:"),
        pick_list(Codec::ALL, Some(app.codec.clone()), Message::CodecSelected).width(150),
        Space::new().width(12),
        text("Format:"),
        pick_list(
            Format::ALL,
            Some(app.format.clone()),
            Message::FormatSelected
        )
        .width(130),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // let region_row: Element<Message> = if app.slurp_installed {
    //     let btn_label = if app.region.is_some() {
    //         "↺ Re-select Region"
    //     } else {
    //         "⬚ Select Region"
    //     };
    //     row![
    //         button(btn_label).on_press(Message::SelectRegion),
    //         Space::new().width(8),
    //         text(
    //             app.region
    //                 .as_deref()
    //                 .unwrap_or("Full screen (no region selected)")
    //         )
    //         .size(13),
    //     ]
    //     .spacing(4)
    //     .align_y(Alignment::Center)
    //     .into()
    // } else {
    //     row![
    //         text("⚠  slurp not found — region selection unavailable").size(13),
    //         Space::new().width(12),
    //         button("Install slurp").on_press(Message::InstallSlurp),
    //     ]
    //     .spacing(8)
    //     .align_y(Alignment::Center)
    //     .into()
    // };
    let region_row: Element<Message> = if app.slurp_installed {
        let icon = Canvas::new(RegionIcon {
            selected: app.region.is_some(),
        })
        .width(16)
        .height(16);

        let btn_label = if app.region.is_some() {
            "Re-select Region"
        } else {
            "Select Region"
        };

        row![
            button(
                row![icon, text(btn_label)]
                    .spacing(8)
                    .align_y(Alignment::Center)
            )
            .padding([8, 16])
            .style(secondary_button_style())
            .on_press(Message::SelectRegion),
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
        let icon = Canvas::new(WarningIcon).width(16).height(16);

        row![
            row![
                icon,
                text("slurp not found — region selection unavailable").size(13)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            Space::new().width(12),
            button("Install slurp")
                .padding([8, 16])
                .style(warning_button_style())
                .on_press(Message::InstallSlurp),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    let record_btn = {
        let icon = Canvas::new(RecordIcon {
            is_recording: app.is_recording,
        })
        .width(16)
        .height(16);

        let label = if app.is_recording {
            "Stop Recording"
        } else {
            "Start Recording"
        };

        button(
            row![icon, text(label)]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .padding([10, 20])
        .style(record_button_style(app.is_recording))
        .on_press(Message::ToggleRecording)
    };

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
