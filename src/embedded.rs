// ========== Embedded Heap Configuration ==========

// Embedded heap configuration for all no_std targets
#[cfg(target_os = "none")]
pub(crate) mod embedded_heap_config {
    use embedded_alloc::Heap;
    #[cfg(not(target_os = "none"))]
    use once_cell::sync::Lazy;

    // Architecture-specific heap sizes based on typical available memory
    // These are conservative defaults that work well for most embedded applications
    // Users can override by defining custom heap sizes in their own code

    #[cfg(target_arch = "avr")]
    pub const HEAP_SIZE: usize = 512; // AVR (Arduino Uno): 2KB total, use 512B heap (25%)

    #[cfg(target_arch = "msp430")]
    pub const HEAP_SIZE: usize = 256; // MSP430: 1KB total, use 256B heap (25%)

    #[cfg(target_arch = "riscv32")]
    pub const HEAP_SIZE: usize = 2048; // RISC-V 32-bit: typically 32KB+, use 2KB heap (6%)

    #[cfg(target_arch = "riscv64")]
    pub const HEAP_SIZE: usize = 4096; // RISC-V 64-bit: typically 128KB+, use 4KB heap (3%)

    #[cfg(target_arch = "xtensa")]
    pub const HEAP_SIZE: usize = 4096; // Xtensa (ESP32): 256KB+, use 4KB heap (1.5%)

    #[cfg(target_arch = "arm")]
    pub const HEAP_SIZE: usize = 1024; // ARM Cortex-M: typically 16KB+, use 1KB heap (6%)

    // Default heap size for other embedded architectures (LoongArch, Hexagon, BPF, SPARC, etc.)
    #[cfg(not(any(
        target_arch = "avr",
        target_arch = "msp430", 
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "xtensa",
        target_arch = "arm"
    )))]
    pub const HEAP_SIZE: usize = 2048; // Conservative default for unknown architectures

    // Static memory pool for embedded heap
    // This is a conservative allocation that should work on most embedded systems
    pub static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    // Singleton heap instance - different implementations for std vs no_std
    #[cfg(not(target_os = "none"))]
    pub static EMBEDDED_HEAP: Lazy<Heap> = Lazy::new(|| unsafe { Heap::new(&mut HEAP_MEMORY[..]) });
    
    #[cfg(target_os = "none")]
    static mut EMBEDDED_HEAP_INSTANCE: Option<Heap> = None;
    
    /// Gets the embedded heap instance for no_std environments
    /// 
    /// This function provides access to the global embedded heap used in no_std 
    /// environments. The heap is lazily initialized on first access with 
    /// architecture-appropriate size defaults.
    /// 
    /// # Returns
    /// 
    /// A reference to the static embedded heap instance
    /// 
    /// # Safety
    /// 
    /// This function is only available in no_std environments (`target_os = "none"`).
    /// The heap initialization is done safely using static guarantees.
    #[cfg(target_os = "none")]
    pub fn get_embedded_heap() -> &'static Heap {
        unsafe {
            if EMBEDDED_HEAP_INSTANCE.is_none() {
                let heap = Heap::empty();
                heap.init(HEAP_MEMORY.as_mut_ptr() as usize, HEAP_SIZE);
                EMBEDDED_HEAP_INSTANCE = Some(heap);
            }
            EMBEDDED_HEAP_INSTANCE.as_ref().unwrap()
        }
    }
}

