//! Library to configure runtime configurations

use std::sync::atomic::Ordering;

use atomic::Atomic;

/// Default redact marker head, used to mark the redacted data.
/// The default value is `‹`. The character U+2039 "‹" is a
/// single-character representation of the left-pointing.
pub const DEFAULT_REDACT_MARKER_HEAD: &str = "‹";
/// Default redact marker tail, used to mark the redacted data.
/// The default value is `›`. The character U+203A "›" is a
/// single-character representation of the right-pointing.
pub const DEFAULT_REDACT_MARKER_TAIL: &str = "›";

/// RedactLevel is used to control the redaction of log data.
///
/// Default is `Off`, means no redaction. And `Marker` is a
/// special flag that replaces sensitive data with a marked placeholder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RedactLevel {
    /// No redaction
    Off,
    /// Redact the data with the '?'.
    On,
    /// Replace the data with `‹?›`.
    Marker, // flag is ‹..›
}

/// In redaction modes, bytes and strings are replaced with a placeholder.
pub(crate) static REDACT_LEVEL: Atomic<RedactLevel> = Atomic::new(RedactLevel::Off);

/// Set redact level.
pub fn set_redact_level(redact_level: RedactLevel) {
    REDACT_LEVEL.store(redact_level, Ordering::Relaxed);
}
