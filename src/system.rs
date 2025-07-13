use crate::types::SystemInfo;
// ========== System Information Collection ==========

#[cfg(not(target_os = "none"))]
pub(crate) fn collect_system_info() -> SystemInfo {
    let total_memory = get_total_memory_safe();
    SystemInfo {
        os_type: std::env::consts::OS.to_string(),
        cpu_cores: std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
        total_memory_bytes: total_memory,
        is_debug: cfg!(debug_assertions),
        is_wasm: cfg!(target_arch = "wasm32"),
        target_arch: std::env::consts::ARCH.to_string(),
    }
}

/// Simplified system info collection for no_std environments
#[cfg(target_os = "none")]
pub(crate) fn collect_system_info() -> SystemInfo {
    let total_memory = get_total_memory_safe();
    SystemInfo {
        os_type: "embedded",
        cpu_cores: 1, // Assume single core for embedded
        total_memory_bytes: total_memory,
        is_debug: cfg!(debug_assertions),
        is_wasm: false,
        target_arch: {
            #[cfg(target_arch = "riscv32")]
            { "riscv32" }
            #[cfg(target_arch = "riscv64")]
            { "riscv64" }
            #[cfg(target_arch = "arm")]
            { "arm" }
            #[cfg(target_arch = "avr")]
            { "avr" }
            #[cfg(target_arch = "msp430")]
            { "msp430" }
            #[cfg(target_arch = "xtensa")]
            { "xtensa" }
            #[cfg(not(any(
                target_arch = "riscv32",
                target_arch = "riscv64", 
                target_arch = "arm",
                target_arch = "avr",
                target_arch = "msp430",
                target_arch = "xtensa"
            )))]
            { "unknown" }
        },
    }
}

/// Detects total system memory without allocating during global allocator initialization
///
/// Uses platform-specific APIs for servers/desktop systems and conservative defaults for embedded platforms.
/// Critical: This function must not allocate memory as it's called during global allocator setup.
#[allow(unreachable_code)]
fn get_total_memory_safe() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        // WASM can dynamically detect memory through core::arch::wasm32
        use core::arch::wasm32;

        // Get current memory pages, each page is 64KB
        let pages = wasm32::memory_size(0); // Memory index 0 is default memory
        let total_bytes = (pages as u64) * 65536;

        return total_bytes;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: use sysctl(HW_MEMSIZE)
        unsafe {
            let mut total_size: u64 = 0;
            let mut mib = [libc::CTL_HW, libc::HW_MEMSIZE];
            let mut len = std::mem::size_of::<u64>();

            if libc::sysctl(
                mib.as_mut_ptr(),
                2,
                &mut total_size as *mut _ as *mut libc::c_void,
                &mut len,
                std::ptr::null_mut(),
                0,
            ) == 0
            {
                return total_size;
            } else {
                return 16u64 << 30; // Fallback: 16GB default
            }
        }
    }

    #[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
    {
        // Linux: use sysinfo() system call
        unsafe {
            let mut info: libc::sysinfo = std::mem::zeroed();
            if libc::sysinfo(&mut info) == 0 {
                let total = info.totalram as u64 * info.mem_unit as u64;
                return total;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        unsafe {
            let mut mem_status: MEMORYSTATUSEX = mem::zeroed();
            mem_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(&mut mem_status) != 0 {
                return mem_status.ullTotalPhys;
            }
        }
    }

    // Embedded platforms: conservative memory size estimates
    #[cfg(target_arch = "avr")]
    {
        return 2u64 << 10; // 2KB for AVR (like Arduino Uno with 2KB RAM)
    }

    #[cfg(target_arch = "msp430")]
    {
        return 1u64 << 10; // 1KB for MSP430 (typical low-power MCU)
    }

    #[cfg(target_arch = "riscv32")]
    {
        return 32u64 << 10; // 32KB for RISC-V MCUs (like ESP32-C3 type devices)
    }

    #[cfg(target_arch = "riscv64")]
    {
        return 128u64 << 10; // 128KB for RISC-V 64-bit systems (like our QEMU example)
    }

    #[cfg(target_arch = "xtensa")]
    {
        return 256u64 << 10; // 256KB for Xtensa (like ESP32 with up to 520KB)
    }

    #[cfg(all(target_arch = "arm", target_os = "none"))]
    {
        return 16u64 << 10; // 16KB for ARM Cortex-M (conservative estimate, M0+ typically has this capacity)
    }

    // Default for unknown platforms
    2u64 << 30
}

