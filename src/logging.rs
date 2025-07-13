#[cfg(unix)] use libc;
use crate::platform::LOG_FLUSHED;
#[cfg(not(target_os = "none"))] use core::sync::atomic::Ordering;
#[cfg(not(target_os = "none"))] use once_cell::sync::Lazy;
#[cfg(not(target_os = "none"))] use log::info;
// ========== Logging System ==========

#[cfg(not(target_os = "none"))]
static PENDING_LOG_MESSAGE: Lazy<std::sync::Mutex<Option<String>>> =
    Lazy::new(|| std::sync::Mutex::new(None));

/// Records allocator selection using a dual logging strategy
///
/// Immediately outputs to stderr (safe during global allocator init) and 
/// saves for later output through the logging framework when available.
#[cfg(not(target_os = "none"))]
pub(crate) fn record_allocator_selection(allocator_name: &str, reason: &str) {
    let message = format!("Auto-allocator: {} selected - {}", allocator_name, reason);

    // Immediate output to stderr (only safe method in global allocator)
    #[cfg(unix)]
    {
        let stderr_message = format!("[INFO] {}\n", message);
        unsafe {
            libc::write(
                2,
                stderr_message.as_ptr() as *const libc::c_void,
                stderr_message.len(),
            );
        }
    }

    // Save message, output later through logging framework
    if let Ok(mut pending) = PENDING_LOG_MESSAGE.lock() {
        *pending = Some(message);
    }
}

/// Attempts to flush pending log message to the logging framework
#[cfg(not(target_os = "none"))]
pub(crate) fn try_flush_pending_log() {
    if !LOG_FLUSHED.load(Ordering::Relaxed) {
        if let Ok(mut pending) = PENDING_LOG_MESSAGE.lock() {
            if let Some(message) = pending.take() {
                let _ = std::panic::catch_unwind(|| {
                    info!("{}", message);
                });
                LOG_FLUSHED.store(true, Ordering::Relaxed);
            }
        }
    }
}

/// Intelligently flushes logs when the logging framework becomes available
#[cfg(not(target_os = "none"))]
pub(crate) fn smart_try_flush_log() {
    // If already output, no need to try again
    if LOG_FLUSHED.load(Ordering::Relaxed) {
        return;
    }

    // Try to output log
    try_flush_pending_log();

    // If still not successful, logging framework is not yet initialized
    // Will continue trying on next call
}

// ========== System Information Collection ==========

#[cfg(target_os = "none")]
pub(crate) fn smart_try_flush_log() {}
