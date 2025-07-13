use core::sync::atomic::{AtomicU8, AtomicBool};
// ========== Platform Detection ==========

/// Checks if the target is an embedded platform requiring specialized allocation
/// 
/// Uses `target_os = "none"` as the primary indicator of embedded/no_std environments.
/// This approach covers all current and future embedded targets automatically,
/// including architectures like RISC-V, ARM, AVR, MSP430, Xtensa, LoongArch, etc.
pub(crate) const fn is_embedded_target() -> bool {
    cfg!(target_os = "none")
}

/// Checks if mimalloc can be used on this platform
pub(crate) const fn can_use_mimalloc() -> bool {
    cfg!(all(
        feature = "_mimalloc",
        any(target_os = "windows", target_os = "macos", target_os = "linux"),
        not(target_arch = "wasm32"),
        not(debug_assertions)
    ))
}

/// Checks if secure mimalloc can be used on this platform
pub(crate) const fn can_use_mimalloc_secure() -> bool {
    cfg!(all(
        feature = "_mimalloc_secure",
        any(target_os = "windows", target_os = "macos", target_os = "linux"),
        not(target_arch = "wasm32"),
        not(debug_assertions)
    ))
}
/// This optimization avoids unnecessary runtime checks for 90% of platforms.
pub(crate) const fn get_compile_time_allocator() -> Option<u8> {
    if is_embedded_target() {
        return Some(4); // embedded-alloc
    }

    if cfg!(target_arch = "wasm32") {
        return Some(1); // system
    }

    if cfg!(debug_assertions) {
        return Some(1); // system (debug builds)
    }

    // Platforms with superior native allocators
    if cfg!(target_os = "android") {
        return Some(1); // Scudo
    }

    if cfg!(target_os = "ios") {
        return Some(1); // libmalloc
    }

    if cfg!(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd")) {
        return Some(1); // native jemalloc/security-hardened
    }

    if cfg!(any(target_os = "solaris", target_os = "illumos")) {
        return Some(1); // libumem
    }

    None // High-performance platforms need runtime detection
}

/// Selects allocator using compile-time rules and runtime hardware detection
pub(crate) fn select_allocator_by_hardware() -> u8 {
    if let Some(allocator_id) = get_compile_time_allocator() {
        return allocator_id;
    }

    // Only high-performance platforms reach here - need CPU core detection
    // Use zero-allocation CPU detection to avoid infinite recursion
    let cpu_cores = get_cpu_cores_safe();

    // Multi-core systems: prefer mimalloc (secure > regular > system)
    if cpu_cores >= 2 && can_use_mimalloc_secure() {
        return 5; // mimalloc-secure
    }

    // Check if mimalloc is available
    // Since build script ensures compatibility, mimalloc is available if feature is enabled
    if cpu_cores >= 2 && can_use_mimalloc() {
        return 2; // mimalloc
    }

    1 // system (single-core or all high-performance allocators unavailable)
}

/// Get CPU core count without allocating memory (to avoid infinite recursion)
pub(crate) fn get_cpu_cores_safe() -> usize {
    #[cfg(unix)]
    {
        // Use direct libc calls to avoid std allocation
        unsafe {
            let cores = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
            if cores > 0 {
                cores as usize
            } else {
                1
            }
        }
    }
    
    #[cfg(windows)]
    {
        // Windows: Use direct WinAPI to avoid std allocation
        use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
        unsafe {
            let mut sysinfo: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut sysinfo);
            sysinfo.dwNumberOfProcessors as usize
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback: assume multi-core for unknown platforms
        4
    }
}

// ========== Embedded Heap Configuration ==========
// ========== Runtime Allocator Selection ==========

// Global state for allocator selection and logging  
// ID mapping: 0=uninitialized, 1=system, 2=mimalloc, 3=jemalloc, 4=embedded, 5=mimalloc-secure
pub(crate) static RUNTIME_ALLOCATOR_ID: AtomicU8 = AtomicU8::new(0);
#[cfg(not(target_os = "none"))]
pub(crate) static ALLOCATOR_LOGGED: AtomicBool = AtomicBool::new(false);
#[cfg(not(target_os = "none"))]
pub(crate) static LOG_FLUSHED: AtomicBool = AtomicBool::new(false);

