//! # Auto Allocator - Zero-Configuration Memory Optimization
//!
//! **Just add one line and get optimal memory performance automatically!**
//!
//! Auto-allocator automatically selects the best memory allocator for your platform and hardware,
//! giving you significant performance improvements without any configuration or code changes.
//!
//! ## Why Auto Allocator?
//!
//! - **🚀 Instant Performance**: Up to 1.6x faster allocation in multi-threaded applications
//! - **🔧 Zero Configuration**: Works perfectly out-of-the-box, no setup required
//! - **🌍 Universal Compatibility**: Optimizes across all platforms - servers, desktop, mobile, embedded, WASM
//! - **🧠 Platform Intelligence**: Automatically chooses the best allocator for each platform
//! - **⚡ Production Ready**: Used safely in high-performance production environments
//!
//! ## Quick Start
//!
//! **Step 1:** Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! auto-allocator = "*"
//! ```
//!
//! **Step 2:** Add one line to your `main.rs`:
//! ```rust,ignore
//! use auto_allocator; // That's it! 🎉
//!
//! fn main() {
//!     // Your code automatically runs with optimal memory performance
//!     let data = vec![1, 2, 3, 4, 5];
//!     println!("Memory allocations are now optimized!");
//! }
//! ```
//!
//! **That's literally all you need!** Auto-allocator handles everything else automatically.
//!
//! ## What You Get
//!
//! - **Linux Servers**: mimalloc for superior multi-threaded performance
//! - **Windows/macOS**: mimalloc for desktop application speed
//! - **Android/iOS**: Platform-optimized system allocators (Scudo/libmalloc)
//! - **Docker/Kubernetes**: Optimized for containerized deployments
//! - **Embedded Systems**: Automatic embedded-alloc for all no_std platforms (RISC-V, ARM, AVR, MSP430, Xtensa, etc.)
//! - **WASM**: Compatible allocation for web applications
//!
//! **Security Mode Available:**
//! ```toml
//! auto-allocator = { version = "*", features = ["secure"] }
//! ```

#![cfg_attr(target_os = "none", no_std)]

mod types;
mod format;
mod platform;
mod embedded;
mod runtime;
mod logging;
mod system;
mod api;

pub use types::{AllocatorInfo, AllocatorType, SystemInfo};
pub use format::format_memory_size;
pub use api::{
    get_allocator_info,
    get_allocator_type,
    get_recommended_allocator,
    check_allocator_optimization,
};
#[cfg(target_arch = "wasm32")]
pub use api::wasm_auto_init;
