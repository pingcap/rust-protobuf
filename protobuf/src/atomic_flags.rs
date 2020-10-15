//! Library to configure runtime configurations

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/// If `REDACT_BYTES` is set, all bytes and strings will be
/// formatted as "?"
pub(crate) static REDACT_BYTES: AtomicBool = AtomicBool::new(false);

/// Set redact bytes.
pub fn set_redact_bytes(redact_bytes: bool) {
    REDACT_BYTES.store(redact_bytes, Ordering::Relaxed);
}
