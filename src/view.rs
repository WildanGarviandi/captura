//! User interface rendering.

use crate::codec::Codec;
use crate::format::Format;
use crate::icons::{RecordIcon, RegionIcon, WarningIcon};
use crate::message::Message;
use crate::model::{App, FPS_OPTIONS};
use crate::styles::{record_button_style, secondary_button_style, warning_button_style};
use iced::widget::{Canvas, Space, button, column, container, pick_list, row, rule, text, text_input};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
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
            Space::new().width(12),
            button("Full Screen")
                .padding([8, 16])
                .style(secondary_button_style())
                .on_press(Message::FullScreen),
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

    if let Some(w) = wf_warning {
        layout = layout.push(w);
    }

    container(layout)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
}
