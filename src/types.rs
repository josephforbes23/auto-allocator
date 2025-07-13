/// 5. **Embedded** (`target_os = "none"`): embedded-alloc (all no_std architectures)
///
/// # Example
///
/// ```rust
/// use auto_allocator;
///
/// let info = auto_allocator::get_allocator_info();
/// match info.allocator_type {
///     auto_allocator::AllocatorType::Mimalloc => {
///         println!("Using mimalloc - optimal performance");
///     }
///     auto_allocator::AllocatorType::System => {
///         println!("Using system allocator - platform compliance");
///     }
///     _ => println!("Using other allocator"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocatorType {

    /// Security-hardened mimalloc allocator
    ///
    /// Microsoft-developed allocator with enhanced security features.
    /// ~10% performance overhead for comprehensive heap protection.
    /// Available when `secure` feature is enabled on compatible platforms.
    MimallocSecure,

    /// High-performance mimalloc allocator
    ///
    /// Microsoft-developed allocator optimized for multi-threaded workloads.
    /// Automatically selected on modern systems with GCC 4.9+ and stdatomic.h.
    Mimalloc,


    /// Embedded systems allocator
    ///
    /// Lightweight allocator designed for resource-constrained environments.
    /// Automatically selected on embedded architectures.
    EmbeddedHeap,

    /// System default allocator
    ///
    /// Operating system provided allocator, maximum compatibility.
    /// Selected for debug builds, WASM, mobile, and platforms with optimized native allocators.
    System,
}

/// Allocator information structure
///
/// Contains the currently selected allocator type, selection reason, and system information.
/// Obtained through the [`get_allocator_info()`] function.
///
/// # Fields
///
/// - `allocator_type` - Currently used allocator type
/// - `reason` - Detailed reason for allocator selection, including hardware information
/// - `system_info` - System hardware and environment information
///
/// # Example
///
/// ```rust
/// use auto_allocator;
///
/// let info = auto_allocator::get_allocator_info();
/// println!("Allocator: {:?}", info.allocator_type);
/// println!("Selection reason: {}", info.reason);
/// println!("CPU cores: {}", info.system_info.cpu_cores);
/// ```
#[derive(Debug, Clone)]
pub struct AllocatorInfo {
    /// Currently used allocator type
    pub allocator_type: AllocatorType,

    /// Detailed reason for allocator selection
    ///
    /// Contains hardware detection results and selection logic explanation, for example:
    /// "mimalloc selected by runtime hardware analysis (16 cores, 128GB total RAM)"
    #[cfg(not(target_os = "none"))]
    pub reason: String,
    #[cfg(target_os = "none")]
    pub reason: &'static str,

    /// System hardware and environment information
    pub system_info: SystemInfo,
}

/// System information structure
///
/// Contains runtime-detected system hardware and environment information,
/// used for allocator selection decisions.
///
/// # Fields
///
/// - `os_type` - Operating system type (linux, macos, windows, etc.)
/// - `cpu_cores` - CPU core count (including hyperthreaded cores)
/// - `total_memory_bytes` - Total memory in bytes
/// - `is_debug` - Whether this is a Debug build
/// - `is_wasm` - Whether this is a WASM environment
/// - `target_arch` - Target architecture (x86_64, aarch64, etc.)
///
/// # Example
///
/// ```rust
/// use auto_allocator;
///
/// let info = auto_allocator::get_allocator_info();
/// let sys = &info.system_info;
///
/// println!("Operating system: {}", sys.os_type);
/// println!("CPU cores: {}", sys.cpu_cores);
/// println!("Total memory: {}", auto_allocator::format_memory_size(sys.total_memory_bytes));
/// ```
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Operating system type
    ///
    /// Examples: "linux", "macos", "windows", "unknown"
    #[cfg(not(target_os = "none"))]
    pub os_type: String,
    #[cfg(target_os = "none")]
    pub os_type: &'static str,

    /// CPU core count
    ///
    /// Detected via `std::thread::available_parallelism()`, includes hyperthreaded core count
    pub cpu_cores: usize,

    /// Total memory in bytes
    ///
    /// System total physical memory, used for hardware specification assessment.
    /// Use [`format_memory_size()`] to format as human-readable string.
    pub total_memory_bytes: u64,

    /// Whether this is a Debug build
    ///
    /// Debug builds automatically select system allocator for faster compilation
    pub is_debug: bool,

    /// Whether this is a WASM environment
    ///
    /// WASM environments automatically select system allocator for compatibility
    pub is_wasm: bool,

    /// Target architecture
    ///
    /// Examples: "x86_64", "aarch64", "riscv32", "wasm32"
    #[cfg(not(target_os = "none"))]
    pub target_arch: String,
    #[cfg(target_os = "none")]
    pub target_arch: &'static str,
}

