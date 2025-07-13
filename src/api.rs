use core::sync::atomic::Ordering;
#[cfg(not(target_os = "none"))] use once_cell::sync::Lazy;
use crate::logging::smart_try_flush_log;
use crate::types::{AllocatorInfo, AllocatorType, SystemInfo};
use crate::platform::{RUNTIME_ALLOCATOR_ID};
use crate::platform::is_embedded_target;
use crate::runtime::RuntimeAllocator;
use crate::system::collect_system_info;
use crate::format::format_memory_size;
#[cfg(not(target_os = "none"))]
static ALLOCATOR_INFO: Lazy<AllocatorInfo> = Lazy::new(|| {
    let system_info = collect_system_info();
    let allocator_id = RUNTIME_ALLOCATOR_ID.load(Ordering::Acquire);

    // If not yet initialized, trigger allocator selection once
    let final_allocator_id = if allocator_id == 0 {
        RuntimeAllocator::get_allocator_id()
    } else {
        allocator_id
    };

    let (_, mut reason) = get_allocator_selection_result(&system_info);

    // Determine type based on actually selected allocator ID (may differ due to feature disable)
    let allocator_type = match final_allocator_id {
        5 => AllocatorType::MimallocSecure,
        2 => AllocatorType::Mimalloc,
        4 => AllocatorType::EmbeddedHeap,
        _ => AllocatorType::System,
    };

    // Add "selected by runtime analysis" prefix to actual allocator info, extract hardware info part
    let hardware_info = if reason.contains('(') && reason.contains(')') {
        reason
            .split_once('(')
            .and_then(|(_prefix, suffix)| suffix.split_once(')').map(|(info, _)| info))
            .unwrap_or("")
    } else {
        ""
    };

    reason = match final_allocator_id {
        5 => format!(
            "mimalloc-secure selected by runtime hardware analysis ({})",
            hardware_info
        ),
        2 => format!(
            "mimalloc selected by runtime hardware analysis ({})",
            hardware_info
        ),
        4 => {
            // For embedded allocator, preserve the original compile-time selection info
            reason
        },
        _ => {
            // For system allocator, preserve the original detailed reason as-is
            // (already includes correct "compile-time selected" or platform-specific info)
            reason
        },
    };

    AllocatorInfo {
        allocator_type,
        reason,
        system_info,
    }
});

// Simplified allocator info for no_std
#[cfg(target_os = "none")]
static mut EMBEDDED_ALLOCATOR_INFO: Option<AllocatorInfo> = None;

// ========== Public API ==========

/// Ensure allocator information is ready
/// Internal function, ensures ALLOCATOR_INFO has been computed
#[cfg(not(target_os = "none"))]
#[cfg(not(target_os = "none"))]
fn ensure_allocator_info_ready() {
    let _ = std::panic::catch_unwind(|| {
        Lazy::force(&ALLOCATOR_INFO);
    });
}
#[cfg(target_os = "none")]
fn ensure_allocator_info_ready() {
    unsafe {
        if EMBEDDED_ALLOCATOR_INFO.is_none() {
            let system_info = collect_system_info();
            EMBEDDED_ALLOCATOR_INFO = Some(AllocatorInfo {
                allocator_type: AllocatorType::EmbeddedHeap,
                reason: "embedded-alloc selected for no_std environment",
                system_info,
            });
        }
    }
}

/// Returns information about the automatically selected allocator
///
/// Provides allocator type, selection rationale, and system information.
/// First call triggers hardware detection; subsequent calls return cached results.
///
/// # Example
///
/// ```rust
/// use auto_allocator;
///
/// let info = auto_allocator::get_allocator_info();
/// println!("Using: {:?}", info.allocator_type);
/// println!("Reason: {}", info.reason);
/// ```
#[cfg(not(target_os = "none"))]
pub fn get_allocator_info() -> &'static AllocatorInfo {
    smart_try_flush_log();
    ensure_allocator_info_ready();
    &ALLOCATOR_INFO
}

#[cfg(target_os = "none")]
pub fn get_allocator_info() -> &'static AllocatorInfo {
    ensure_allocator_info_ready();
    unsafe { EMBEDDED_ALLOCATOR_INFO.as_ref().unwrap() }
}

/// Get current allocator type
///
/// Returns the currently used allocator type, this is a simplified version of [`get_allocator_info()`].
/// If you only need to know the allocator type without other information, using this function is more concise.
///
/// # Return Value
///
/// Returns [`AllocatorType`] enum value, possible values:
/// - [`AllocatorType::Mimalloc`] - Microsoft-developed high-performance allocator
/// - [`AllocatorType::EmbeddedHeap`] - Embedded systems specific allocator
/// - [`AllocatorType::System`] - System default allocator
///
/// # Example
///
/// ```rust
/// use auto_allocator;
///
/// let allocator_type = auto_allocator::get_allocator_type();
///
/// // Simple allocator type check
/// if allocator_type == auto_allocator::AllocatorType::Mimalloc {
///     println!("Using high-performance mimalloc allocator");
/// }
///
/// // Or use match statement
/// match allocator_type {
///     auto_allocator::AllocatorType::Mimalloc => {
///         println!("mimalloc - optimal performance");
///     }
///     auto_allocator::AllocatorType::System => {
///         println!("system - maximum compatibility");
///     }
///     _ => println!("other allocator"),
/// }
/// ```
///
/// # Performance Notes
///
/// This function is slightly faster than [`get_allocator_info()`] because it only returns type information.
pub fn get_allocator_type() -> AllocatorType {
    smart_try_flush_log();
    ensure_allocator_info_ready();
    get_allocator_info().allocator_type
}

/// Get allocator selection result and reason (internal function)
#[cfg(not(target_os = "none"))]
fn get_allocator_selection_result(system_info: &SystemInfo) -> (AllocatorType, String) {
    let total_mem = format_memory_size(system_info.total_memory_bytes);

    if system_info.is_wasm {
        (
            AllocatorType::System,
            format!("system allocator - WASM environment ({} total RAM)", total_mem),
        )
    } else if system_info.is_debug {
        (
            AllocatorType::System,
            format!(
                "system allocator - debug build ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    } else if is_embedded_target() {
        (
            AllocatorType::EmbeddedHeap,
            format!("embedded-alloc allocator - embedded environment ({} total RAM)", total_mem),
        )
    } else if system_info.os_type == "android" {
        (
            AllocatorType::System,
            format!(
                "Android platform - Scudo allocator (security-first, use-after-free protection) ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    } else if system_info.os_type == "ios" {
        (
            AllocatorType::System,
            format!(
                "iOS platform - libmalloc allocator (Apple-optimized, memory pressure handling) ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    } else if system_info.os_type == "freebsd" || system_info.os_type == "netbsd" {
        (
            AllocatorType::System,
            format!(
                "BSD platform - native jemalloc (highly optimized, deep system integration) ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    } else if system_info.os_type == "openbsd" {
        (
            AllocatorType::System,
            format!(
                "OpenBSD platform - security-hardened allocator (exploit mitigation, aggressive hardening) ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    } else if system_info.os_type == "solaris" || system_info.os_type == "illumos" {
        (
            AllocatorType::System,
            format!(
                "Solaris platform - libumem allocator (NUMA-aware, enterprise-grade performance) ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    } else if system_info.cpu_cores >= 2 {
        (
            AllocatorType::Mimalloc,
            format!(
                "mimalloc allocator - high-performance multi-threaded environment ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    } else {
        (
            AllocatorType::System,
            format!(
                "system allocator - low-performance environment ({} cores, {} total RAM)",
                system_info.cpu_cores, total_mem
            ),
        )
    }
}

/// Simplified allocator selection for no_std environments
#[cfg(target_os = "none")]
fn get_allocator_selection_result(_system_info: &SystemInfo) -> (AllocatorType, &'static str) {
    (AllocatorType::EmbeddedHeap, "embedded-alloc selected for no_std environment")
}

/// Get recommended allocator for current runtime environment
///
/// Based on current system hardware and environment re-analysis, returns recommended allocator type and selection reason.
/// Unlike [`get_allocator_info()`], this function re-performs hardware detection and analysis every time.
///
/// # Return Value
///
/// Returns a tuple `(AllocatorType, String)`:
/// - First element: recommended allocator type
/// - Second element: recommendation reason, including hardware information
///
/// # Usage
///
/// This function is mainly used for:
/// - Performance analysis and optimization recommendations
/// - Verifying if current allocator selection is optimal
/// - Re-evaluation after runtime environment changes
///
/// # Examples
///
/// ```rust
/// use auto_allocator;
///
/// let (recommended_type, reason) = auto_allocator::get_recommended_allocator();
///
/// println!("Recommended allocator: {:?}", recommended_type);
/// println!("Recommendation reason: {}", reason);
///
/// // Compare with current allocator
/// let current_type = auto_allocator::get_allocator_type();
/// if current_type == recommended_type {
///     println!("Current allocator is already optimal");
/// } else {
///     println!("Suggest switching to: {:?}", recommended_type);
/// }
/// ```
///
/// # Performance Notes
///
/// This function re-performs system hardware detection, with slightly higher overhead than [`get_allocator_info()`].
#[cfg(not(target_os = "none"))]
pub fn get_recommended_allocator() -> (AllocatorType, String) {
    smart_try_flush_log();
    let system_info = collect_system_info();
    get_allocator_selection_result(&system_info)
}

#[cfg(target_os = "none")]
pub fn get_recommended_allocator() -> (AllocatorType, &'static str) {
    let system_info = collect_system_info();
    get_allocator_selection_result(&system_info)
}

/// Check if current allocator is optimal for current environment
///
/// Compares currently used allocator with hardware environment recommended allocator,
/// determining if the best allocator has already been selected.
/// Used for performance optimization checks and configuration validation.
///
/// # Return Value
///
/// Returns a tuple `(bool, Option<String>)`:
/// - `(true, None)` - Current allocator is already optimal
/// - `(false, Some(suggestion))` - Current allocator is not optimal, includes optimization suggestion
///
/// # Usage
///
/// - **Performance audit** - Check if application uses optimal allocator
/// - **Environment validation** - Confirm allocator configuration in deployment environment
/// - **Optimization suggestions** - Get specific allocator optimization recommendations
/// - **Monitoring integration** - Integrate into monitoring systems to check configuration drift
///
/// # Examples
///
/// ```rust
/// use auto_allocator;
///
/// let (is_optimal, suggestion) = auto_allocator::check_allocator_optimization();
///
/// if is_optimal {
///     println!("✅ Current allocator configuration is optimal");
/// } else if let Some(advice) = suggestion {
///     println!("⚠️  Allocator configuration can be optimized:");
///     println!("   {}", advice);
/// }
/// ```
///
/// # Practical Application Scenarios
///
/// ```rust
/// use auto_allocator;
///
/// // Check allocator configuration at application startup
/// fn check_performance_config() {
///     let (is_optimal, suggestion) = auto_allocator::check_allocator_optimization();
///     
///     if !is_optimal {
///         eprintln!("Warning: {}", suggestion.unwrap_or_default());
///         eprintln!("Recommend compiling in Release mode for optimal performance");
///     }
/// }
///
/// // Validate configuration in CI/CD
/// fn test_allocator_optimization() {
///     let (is_optimal, _) = auto_allocator::check_allocator_optimization();
///     assert!(is_optimal, "Allocator configuration not optimized to best state");
/// }
/// ```
///
/// # Performance Notes
///
/// This function needs to re-detect hardware and compare allocators, with slightly higher overhead than simple information retrieval functions.
#[cfg(not(target_os = "none"))]
pub fn check_allocator_optimization() -> (bool, Option<String>) {
    smart_try_flush_log();
    let current = get_allocator_type();
    let (recommended, reason) = get_recommended_allocator();

    if current == recommended {
        (true, None)
    } else {
        let suggestion = format!(
            "Current: {:?}, Recommended: {:?} ({})",
            current, recommended, reason
        );
        (false, Some(suggestion))
    }
}

#[cfg(target_os = "none")]
pub fn check_allocator_optimization() -> (bool, Option<&'static str>) {
    // In no_std, always optimal (embedded-alloc)
    (true, None)
}

// WASM environment initialization
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Automatically initializes allocator information when WASM module loads
///
/// This function is called automatically via `#[wasm_bindgen(start)]` - no manual invocation needed.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_auto_init() {
    ensure_allocator_info_ready();
}

