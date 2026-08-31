//! Video codec enumeration and its ffmpeg/wf-recorder mapping.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Codec {
    H264,
    H265,
    VP8,
    VP9,
    AV1,
}

impl fmt::Display for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    pub const ALL: &'static [Codec] = &[Codec::H264, Codec::H265, Codec::VP8, Codec::VP9, Codec::AV1];

    /// The `-c` argument passed to `wf-recorder`.
    pub fn wf_arg(&self) -> &'static str {
        match self {
            Codec::H264 => "libx264",
            Codec::H265 => "libx265",
            Codec::VP8 => "libvpx",
            Codec::VP9 => "libvpx-vp9",
            Codec::AV1 => "libaom-av1",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_implementation() {
        assert_eq!(format!("{}", Codec::H264), "H.264");
        assert_eq!(format!("{}", Codec::H265), "H.265 / HEVC");
        assert_eq!(format!("{}", Codec::VP8), "VP8");
        assert_eq!(format!("{}", Codec::VP9), "VP9");
        assert_eq!(format!("{}", Codec::AV1), "AV1");
    }

    #[test]
    fn test_wf_arg() {
        assert_eq!(Codec::H264.wf_arg(), "libx264");
        assert_eq!(Codec::H265.wf_arg(), "libx265");
        assert_eq!(Codec::VP8.wf_arg(), "libvpx");
        assert_eq!(Codec::VP9.wf_arg(), "libvpx-vp9");
        assert_eq!(Codec::AV1.wf_arg(), "libaom-av1");
    }
}
