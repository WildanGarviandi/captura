//! Output container format enumeration.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format {
    MP4,
    MKV,
    WebM,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::MP4 => write!(f, "MP4  (.mp4)"),
            Format::MKV => write!(f, "MKV  (.mkv)"),
            Format::WebM => write!(f, "WebM (.webm)"),
        }
    }
}

impl Format {
    pub const ALL: &'static [Format] = &[Format::MP4, Format::MKV, Format::WebM];

    /// The file extension for this format.
    pub fn ext(&self) -> &'static str {
        match self {
            Format::MP4 => "mp4",
            Format::MKV => "mkv",
            Format::WebM => "webm",
        }
    }
}