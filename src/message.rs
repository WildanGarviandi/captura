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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_debug() {
        let message = Message::FpsSelected(30);
        let debug_output = format!("{:?}", message);
        assert!(!debug_output.is_empty());
    }

    #[test]
    fn test_all_message_variants() {
        // Test that all variants can be constructed
        let _ = Message::FpsSelected(30);
        let _ = Message::OutputDirChanged("test".to_string());
        let _ = Message::FilenameChanged("test".to_string());
        let _ = Message::BrowseDir;
        let _ = Message::DirSelected(Some("test".to_string()));
        let _ = Message::CodecSelected(Codec::H264);
        let _ = Message::FormatSelected(Format::MP4);
        let _ = Message::SelectRegion;
        let _ = Message::RegionSelected(Ok("test".to_string()));
        let _ = Message::ToggleRecording;
        let _ = Message::FullScreen;
        let _ = Message::InstallSlurp;
        let _ = Message::SlurpInstalled(Ok(()));
        let _ = Message::Tick;
    }
}
