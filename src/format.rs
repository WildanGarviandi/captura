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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Format::MP4), "MP4  (.mp4)");
        assert_eq!(format!("{}", Format::MKV), "MKV  (.mkv)");
        assert_eq!(format!("{}", Format::WebM), "WebM (.webm)");
    }

    #[test]
    fn test_all() {
        assert_eq!(Format::ALL.len(), 3);
        assert!(Format::ALL.contains(&Format::MP4));
        assert!(Format::ALL.contains(&Format::MKV));
        assert!(Format::ALL.contains(&Format::WebM));
    }

    #[test]
    fn test_ext() {
        assert_eq!(Format::MP4.ext(), "mp4");
        assert_eq!(Format::MKV.ext(), "mkv");
        assert_eq!(Format::WebM.ext(), "webm");
    }
}
