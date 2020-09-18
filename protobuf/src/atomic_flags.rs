//! Library to configure runtime configurations

use std::sync::atomic::AtomicBool;

/// If `REDACT_BYTES` is set, all bytes and strings will be
/// formatted as "???"
pub static REDACT_BYTES: AtomicBool = AtomicBool::new(false);
