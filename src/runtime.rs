#[cfg(not(target_os = "none"))] use std::alloc;
#[cfg(target_os = "none")] use crate::embedded::embedded_heap_config;
use core::sync::atomic::Ordering;
use core::alloc::{GlobalAlloc, Layout};
use crate::platform::{RUNTIME_ALLOCATOR_ID, ALLOCATOR_LOGGED, select_allocator_by_hardware};
use crate::system::collect_system_info;
use crate::logging::record_allocator_selection;
use crate::format::format_memory_size;
// ========== Safe Runtime Allocator Implementation ==========

pub struct RuntimeAllocator;

impl RuntimeAllocator {
    #[inline]
    pub(crate) fn get_allocator_id() -> u8 {
        let current_id = RUNTIME_ALLOCATOR_ID.load(Ordering::Acquire);

        if unlikely(current_id == 0) {
            // First call, perform hardware detection and selection
            let selected_id = select_allocator_by_hardware();
            RUNTIME_ALLOCATOR_ID.store(selected_id, Ordering::Release);

            // Record selection information (ensure only logged once)
            Self::log_allocator_selection(selected_id);

            selected_id
        } else {
            current_id
        }
    }

    #[cold]
    #[cfg(not(target_os = "none"))]
    fn log_allocator_selection(allocator_id: u8) {
        if ALLOCATOR_LOGGED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let (name, reason) = Self::get_allocator_log_info(allocator_id);
            record_allocator_selection(name, &reason);
        }
    }

    #[cold]
    #[cfg(target_os = "none")]
    fn log_allocator_selection(_allocator_id: u8) {
        // No logging in no_std environments
    }

    /// Get logging information based on allocator ID and compile-time platform detection
    #[cfg(not(target_os = "none"))]
    fn get_allocator_log_info(allocator_id: u8) -> (&'static str, String) {
        match allocator_id {
            5 => {
                let system_info = collect_system_info();
                ("mimalloc-secure", format!(
                    "security-hardened choice - runtime detected ({} cores, {} total RAM)",
                    system_info.cpu_cores,
                    format_memory_size(system_info.total_memory_bytes)
                ))
            },
            2 => {
                let system_info = collect_system_info();
                ("mimalloc", format!(
                    "optimal performance choice - runtime detected ({} cores, {} total RAM)",
                    system_info.cpu_cores,
                    format_memory_size(system_info.total_memory_bytes)
                ))
            },
            4 => {
                let system_info = collect_system_info();
                ("embedded-alloc", format!(
                    "embedded platform - compile-time selected ({} total RAM)",
                    format_memory_size(system_info.total_memory_bytes)
                ))
            },
            _ => {
                // System allocator - determine reason based on compile-time platform detection
                if cfg!(debug_assertions) {
                    let system_info = collect_system_info();
                    ("system", format!(
                        "debug build - compile-time selected ({} cores, {} total RAM)",
                        system_info.cpu_cores,
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                } else if cfg!(target_arch = "wasm32") {
                    let system_info = collect_system_info();
                    ("system", format!(
                        "WASM environment - compile-time selected ({} total RAM)",
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                } else if cfg!(target_os = "android") {
                    let system_info = collect_system_info();
                    ("system", format!(
                        "Android Scudo allocator - compile-time selected (security-first policy) ({} cores, {} total RAM)",
                        system_info.cpu_cores,
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                } else if cfg!(target_os = "ios") {
                    let system_info = collect_system_info();
                    ("system", format!(
                        "iOS libmalloc allocator - compile-time selected (Apple optimized) ({} cores, {} total RAM)",
                        system_info.cpu_cores,
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                } else if cfg!(any(target_os = "freebsd", target_os = "netbsd")) {
                    let system_info = collect_system_info();
                    ("system", format!(
                        "BSD native jemalloc - compile-time selected (platform optimized) ({} cores, {} total RAM)",
                        system_info.cpu_cores,
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                } else if cfg!(target_os = "openbsd") {
                    let system_info = collect_system_info();
                    ("system", format!(
                        "OpenBSD security-hardened allocator - compile-time selected ({} cores, {} total RAM)",
                        system_info.cpu_cores,
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                } else if cfg!(any(target_os = "solaris", target_os = "illumos")) {
                    let system_info = collect_system_info();
                    ("system", format!(
                        "Solaris libumem allocator - compile-time selected (enterprise grade) ({} cores, {} total RAM)",
                        system_info.cpu_cores,
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                } else {
                    // High-performance platforms that fell back to system (single-core or mimalloc unavailable)
                    let system_info = collect_system_info();
                    ("system", format!(
                        "runtime fallback - single-core or mimalloc unavailable ({} cores, {} total RAM)",
                        system_info.cpu_cores,
                        format_memory_size(system_info.total_memory_bytes)
                    ))
                }
            },
        }
    }
}

// Branch prediction optimization
#[inline(always)]
fn unlikely(b: bool) -> bool {
    #[cold]
    fn cold() {}
    if b {
        cold();
    }
    b
}

// ========== Global Allocator Implementation - Platform-specific VTable handling ==========

unsafe impl GlobalAlloc for RuntimeAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match Self::get_allocator_id() {

            // mimalloc-secure - security-hardened allocator with 10% performance overhead
            #[cfg(all(
                feature = "_mimalloc_secure",
                not(target_arch = "wasm32"),
                not(debug_assertions),
                not(target_os = "none")
            ))]
            5 => {
                use mimalloc::MiMalloc;
                MiMalloc.alloc(layout)
            }

            // mimalloc - high-performance allocator with compiler compatibility detection
            #[cfg(all(
                feature = "_mimalloc",
                not(target_arch = "wasm32"),
                not(debug_assertions),
                not(target_os = "none")
            ))]
            2 => {
                use mimalloc::MiMalloc;
                MiMalloc.alloc(layout)
            }

            // embedded-alloc - for all no_std embedded platforms
            #[cfg(all(
                feature = "_embedded",
                target_os = "none"
            ))]
            4 => {
                // Use embedded-alloc for all no_std targets
                #[cfg(not(target_os = "none"))]
                {
                    embedded_heap_config::EMBEDDED_HEAP.alloc(layout)
                }
                #[cfg(target_os = "none")]
                {
                    embedded_heap_config::get_embedded_heap().alloc(layout)
                }
            }

            // System allocator - default fallback
            #[cfg(not(target_os = "none"))]
            _ => alloc::System.alloc(layout),
            
            #[cfg(target_os = "none")]
            _ => core::ptr::null_mut(),
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match Self::get_allocator_id() {

            // mimalloc-secure - security-hardened allocator
            #[cfg(all(
                feature = "_mimalloc_secure",
                not(target_arch = "wasm32"),
                not(debug_assertions),
                not(target_os = "none")
            ))]
            5 => {
                use mimalloc::MiMalloc;
                MiMalloc.dealloc(ptr, layout)
            }

            // mimalloc - high-performance allocator with compiler compatibility detection
            #[cfg(all(
                feature = "_mimalloc",
                not(target_arch = "wasm32"),
                not(debug_assertions),
                not(target_os = "none")
            ))]
            2 => {
                use mimalloc::MiMalloc;
                MiMalloc.dealloc(ptr, layout)
            }

            #[cfg(all(
                feature = "_embedded",
                target_os = "none"
            ))]
            4 => {
                // Use embedded-alloc for all no_std targets
                #[cfg(not(target_os = "none"))]
                {
                    embedded_heap_config::EMBEDDED_HEAP.dealloc(ptr, layout)
                }
                #[cfg(target_os = "none")]
                {
                    embedded_heap_config::get_embedded_heap().dealloc(ptr, layout)
                }
            }

            #[cfg(not(target_os = "none"))]
            _ => alloc::System.dealloc(ptr, layout),
            
            #[cfg(target_os = "none")]
            _ => {},
        }
    }
}

#[global_allocator]
static GLOBAL: RuntimeAllocator = RuntimeAllocator;

// ========== Logging System ==========
