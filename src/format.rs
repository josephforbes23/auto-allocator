// ========== Memory Formatting Utilities ==========

/// High-performance memory size formatting function
///
/// Converts byte count to human-readable memory size string, automatically selecting appropriate units.
/// Uses bit shift operations for performance optimization, supports memory sizes from bytes to PB level.
///
/// # Arguments
///
/// - `bytes` - The number of bytes to format
///
/// # Returns
///
/// Returns formatted string, for example:
/// - `1024` → `"1KB"`
/// - `1536` → `"1.5KB"`
/// - `1073741824` → `"1GB"`
///
/// # Supported Units
///
/// - **B** - Bytes (< 1024)
/// - **KB** - Kilobytes (1024 B)
/// - **MB** - Megabytes (1024 KB)
/// - **GB** - Gigabytes (1024 MB)
/// - **TB** - Terabytes (1024 GB)
/// - **PB** - Petabytes (1024 TB)
///
/// # Performance Features
///
/// - Uses bit shift operations instead of division for performance optimization
/// - Hardware-optimized leading zero count instructions
/// - Retains only 1 decimal place for improved performance
/// - Zero-copy string construction
///
/// # Examples
///
/// ```rust
/// use auto_allocator;
///
/// // Basic usage
/// assert_eq!(auto_allocator::format_memory_size(0), "0B");
/// assert_eq!(auto_allocator::format_memory_size(1024), "1KB");
/// assert_eq!(auto_allocator::format_memory_size(1536), "1.5KB");
/// assert_eq!(auto_allocator::format_memory_size(1048576), "1MB");
/// assert_eq!(auto_allocator::format_memory_size(1073741824), "1GB");
///
/// // Use in combination with system information
/// let info = auto_allocator::get_allocator_info();
/// let memory_str = auto_allocator::format_memory_size(info.system_info.total_memory_bytes);
/// println!("Total system memory: {}", memory_str);
///
/// // Display memory usage in application
/// fn display_memory_usage() {
///     let info = auto_allocator::get_allocator_info();
///     println!("Memory information:");
///     println!("  Total memory: {}", auto_allocator::format_memory_size(info.system_info.total_memory_bytes));
/// }
/// ```
///
/// # Precision Notes
///
/// For performance considerations, decimal places are limited to 1 digit. For scenarios
/// requiring higher precision, it is recommended to calculate directly using byte counts.
#[cfg(not(target_os = "none"))]
pub fn format_memory_size(bytes: u64) -> String {
    use std::format;
    
    if bytes == 0 {
        return "0B".to_string();
    }

    // Use bit shift calculations to avoid division operations for performance improvement
    // Each unit has a 1024x relationship, i.e., 2^10
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

    // Use leading zero count to quickly determine appropriate unit level
    // leading_zeros() is a hardware-optimized instruction
    let unit_index = if bytes >= (1u64 << 50) {
        5
    }
    // >= 1PB
    else if bytes >= (1u64 << 40) {
        4
    }
    // >= 1TB
    else if bytes >= (1u64 << 30) {
        3
    }
    // >= 1GB
    else if bytes >= (1u64 << 20) {
        2
    }
    // >= 1MB
    else if bytes >= (1u64 << 10) {
        1
    }
    // >= 1KB
    else {
        0
    }; // < 1KB

    if unit_index == 0 {
        format!("{}B", bytes)
    } else {
        let shift = unit_index * 10; // Each unit is 2^10
        let value = bytes >> shift;
        let remainder = bytes & ((1u64 << shift) - 1);

        // Calculate decimal part (retain only 1 decimal place for performance)
        if remainder == 0 {
            format!("{}{}", value, UNITS[unit_index])
        } else {
            let fraction = (remainder * 10) >> shift;
            if fraction == 0 {
                format!("{}{}", value, UNITS[unit_index])
            } else {
                format!("{}.{}{}", value, fraction, UNITS[unit_index])
            }
        }
    }
}

/// Simplified memory size formatting for no_std environments
#[cfg(target_os = "none")]
pub fn format_memory_size(bytes: u64) -> &'static str {
    // For embedded systems, use predefined size categories
    if bytes == 0 {
        "0B"
    } else if bytes < 1024 {
        "<1KB"
    } else if bytes < (1024 * 1024) {
        "~KB"
    } else if bytes < (1024 * 1024 * 1024) {
        "~MB"
    } else {
        "~GB"
    }
}

// ========== Platform Detection ==========
