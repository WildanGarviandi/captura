//! Application messages — the events that drive state transitions.

use crate::codec::Codec;
use crate::format::Format;

#[derive(Debug, Clone)]
pub enum Message {
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
    FullScreen,
    InstallSlurp,
    SlurpInstalled(Result<(), String>),
    /// Fired every ~500ms by a subscription to check if the notification
    /// stop button was clicked.
    Tick,
}
