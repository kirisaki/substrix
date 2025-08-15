// src/arch.rs
//! Hardware Abstraction Layer (HAL) Foundation
//!
//! This module provides a hardware-independent interface for kernel operations,
//! enabling portability across different architectures while maintaining
//! performance and safety.

// Architecture-specific implementation selection
#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64 as current;

/// Memory address type, sized appropriately for the target architecture
pub type Address = usize;

/// Machine word type, representing the natural word size of the architecture
pub type Word = usize;

/// Hardware register type for control and status registers
pub type Register = usize;

/// Control and Status Register abstraction
///
/// Provides a safe interface for reading and writing architecture-specific
/// control registers while maintaining type safety and access control.
pub trait ControlStatusRegister {
    /// Read the current value of the CSR
    ///
    /// # Returns
    /// The current register value
    fn read(&self) -> Register;

    /// Write a value to the CSR
    ///
    /// # Arguments
    /// * `value` - The value to write to the register
    ///
    /// # Safety
    /// This function is unsafe because writing to control registers can
    /// affect system state, interrupt handling, and memory protection.
    unsafe fn write(&self, value: Register);
}

/// Hardware interrupt controller abstraction
///
/// Provides a unified interface for managing interrupts across different
/// architectures, including enable/disable operations and status queries.
pub trait InterruptController {
    /// Error type for interrupt controller operations
    type Error;

    /// Enable interrupts for this controller
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the operation failed
    ///
    /// # Safety
    /// This function is unsafe because enabling interrupts can affect
    /// system timing and concurrency behavior.
    unsafe fn enable(&self) -> Result<(), Self::Error>;

    /// Disable interrupts for this controller
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the operation failed
    ///
    /// # Safety
    /// This function is unsafe because disabling interrupts can affect
    /// system responsiveness and real-time behavior.
    unsafe fn disable(&self) -> Result<(), Self::Error>;

    /// Check if interrupts are currently enabled
    ///
    /// # Returns
    /// `true` if interrupts are enabled, `false` otherwise
    fn is_enabled(&self) -> bool;
}

/// Hardware timer abstraction
///
/// Provides a unified interface for timer operations, including reading
/// current time and setting alarms for future events.
pub trait Timer {
    /// Error type for timer operations
    type Error;

    /// Duration type representing time intervals
    type Duration;

    /// Get the current time from this timer
    ///
    /// # Returns
    /// The current time value in timer-specific units
    fn now(&self) -> Self::Duration;

    /// Set an alarm to trigger at the specified time
    ///
    /// # Arguments
    /// * `when` - The time at which the alarm should trigger
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the operation failed
    ///
    /// # Safety
    /// This function is unsafe because timer interrupts can affect
    /// system scheduling and real-time behavior.
    unsafe fn set_alarm(&self, when: Self::Duration) -> Result<(), Self::Error>;

    /// Stop the timer and cancel any pending alarms
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the operation failed
    ///
    /// # Safety
    /// This function is unsafe because stopping timers can affect
    /// system scheduling and watchdog functionality.
    unsafe fn stop(&self) -> Result<(), Self::Error>;
}

/// Trap (exception/interrupt) handler abstraction
///
/// Provides a unified interface for handling processor traps, including
/// exceptions and interrupts, with architecture-specific context preservation.
pub trait TrapHandler {
    /// Context type containing processor state during trap handling
    type Context;

    /// Register a trap handler function
    ///
    /// # Arguments
    /// * `handler` - Address of the handler function
    ///
    /// # Returns
    /// `Ok(())` on success, or an error string if registration failed
    ///
    /// # Safety
    /// This function is unsafe because registering trap handlers affects
    /// system exception handling and control flow.
    unsafe fn register(&self, handler: Address) -> Result<(), &'static str>;

    /// Get the current trap context
    ///
    /// # Returns
    /// A snapshot of the processor context at trap time
    fn get_context(&self) -> Self::Context;
}

/// Re-export current architecture's CSR module for compatibility
///
/// This allows existing code to continue using `arch::csr::*` while
/// we transition to the new HAL structure.
pub mod csr {
    pub use super::current::csr::*;
}

/// Architecture information structure
///
/// Contains compile-time constants describing the target architecture's
/// capabilities and characteristics.
pub struct ArchInfo {
    /// Human-readable architecture name
    pub name: &'static str,

    /// Size of a machine word in bytes
    pub word_size: usize,

    /// Memory page size in bytes (if paging is supported)
    pub page_size: usize,

    /// Whether this architecture has a Memory Management Unit
    pub has_mmu: bool,
}

/// Compile-time architecture information
///
/// This constant provides information about the target architecture
/// that can be used for conditional compilation and runtime decisions.
pub const ARCH_INFO: ArchInfo = ArchInfo {
    name: "RISC-V 64-bit",
    word_size: 8,
    page_size: 4096,
    has_mmu: false, // Not currently used in our bare-metal implementation
};

/// Print architecture information to console
///
/// Displays detailed information about the current architecture,
/// including word size, page size, and feature availability.
/// Useful for debugging and system introspection.
pub fn print_arch_info() {
    crate::println!("Architecture Information:");
    crate::println!("  Name: {}", crate::console::FormatArg::Str(ARCH_INFO.name));
    crate::println_number!("  Word size: ", ARCH_INFO.word_size as u64);
    crate::println!(" bytes");
    crate::println_number!("  Page size: ", ARCH_INFO.page_size as u64);
    crate::println!(" bytes");
    if ARCH_INFO.has_mmu {
        crate::println!("  MMU: Available");
    } else {
        crate::println!("  MMU: Not used");
    }
}
