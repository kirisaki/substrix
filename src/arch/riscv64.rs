// src/arch/riscv64/mod.rs
//! RISC-V 64-bit Architecture Implementation
//!
//! This module contains RISC-V specific implementations of the hardware
//! abstraction layer traits, providing direct access to RISC-V control
//! and status registers, interrupt controllers, and timer facilities.

pub mod csr;
pub mod timer;

// Re-export commonly used types for convenience
pub use timer::{ClintTimer, TimerDuration, CLINT_TIMER};

/// Machine word size for RISC-V 64-bit architecture
pub const WORD_SIZE: usize = 8;

/// Standard page size for RISC-V architecture
pub const PAGE_SIZE: usize = 4096;

/// Memory map definitions for QEMU virt machine
///
/// These constants define the physical memory layout used by the QEMU
/// RISC-V virt machine, including RAM regions and memory-mapped peripherals.
pub mod memory_map {
    /// Start address of main RAM
    pub const RAM_START: usize = 0x80000000;

    /// Total RAM size (128 MB)
    pub const RAM_SIZE: usize = 128 * 1024 * 1024;

    /// End address of main RAM
    pub const RAM_END: usize = RAM_START + RAM_SIZE;

    /// UART0 base address for console I/O
    pub const UART0_BASE: usize = 0x10000000;

    /// Core-Local Interruptor (CLINT) base address
    pub const CLINT_BASE: usize = 0x2000000;

    /// CLINT address space size
    pub const CLINT_SIZE: usize = 0x10000;

    /// Machine Software Interrupt Pending register base
    pub const MSIP_BASE: usize = CLINT_BASE + 0x0;

    /// Machine Timer Compare register base
    pub const MTIMECMP_BASE: usize = CLINT_BASE + 0x4000;

    /// Machine Time register address
    pub const MTIME_ADDR: usize = CLINT_BASE + 0xBFF8;
}

/// RISC-V specific error types
///
/// These errors represent various failure modes specific to RISC-V
/// hardware operations and privilege level violations.
#[derive(Debug, Clone, Copy)]
pub enum RiscvError {
    /// Invalid Control and Status Register access attempt
    InvalidCsrAccess,

    /// Memory address is invalid or out of bounds
    InvalidAddress,

    /// Operation requires higher privilege level
    InvalidPrivilege,

    /// Hardware fault or malfunction detected
    HardwareFault,
}

impl core::fmt::Display for RiscvError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            RiscvError::InvalidCsrAccess => write!(f, "Invalid CSR access"),
            RiscvError::InvalidAddress => write!(f, "Invalid address"),
            RiscvError::InvalidPrivilege => write!(f, "Invalid privilege level"),
            RiscvError::HardwareFault => write!(f, "Hardware fault"),
        }
    }
}

/// RISC-V processor context
///
/// Captures the state of important RISC-V control and status registers
/// at a specific point in time, typically during trap handling or
/// system state inspection.
#[derive(Debug, Clone, Copy)]
pub struct RiscvContext {
    /// Machine Status register
    pub mstatus: usize,

    /// Machine Cause register (trap cause)
    pub mcause: usize,

    /// Machine Exception Program Counter
    pub mepc: usize,

    /// Machine Trap Vector Base Address
    pub mtvec: usize,

    /// Machine Interrupt Enable register
    pub mie: usize,

    /// Machine Interrupt Pending register
    pub mip: usize,
}

impl RiscvContext {
    /// Capture the current processor context
    ///
    /// Reads all relevant CSRs and returns a snapshot of the current
    /// processor state. This is useful for debugging and trap analysis.
    ///
    /// # Returns
    /// A `RiscvContext` containing the current CSR values
    pub fn capture() -> Self {
        Self {
            mstatus: csr::read_mstatus(),
            mcause: csr::read_mcause(),
            mepc: csr::read_mepc(),
            mtvec: csr::read_mtvec(),
            mie: csr::read_mie(),
            mip: csr::read_mip(),
        }
    }

    /// Check if the cause register indicates an interrupt
    ///
    /// # Returns
    /// `true` if the trap was caused by an interrupt, `false` for exceptions
    pub fn is_interrupt(&self) -> bool {
        (self.mcause >> 63) != 0
    }

    /// Extract the exception/interrupt code from mcause
    ///
    /// # Returns
    /// The exception or interrupt code (without the interrupt bit)
    pub fn exception_code(&self) -> usize {
        self.mcause & 0x7FFFFFFFFFFFFFFF
    }

    /// Check if global interrupts are enabled
    ///
    /// # Returns
    /// `true` if global interrupts are enabled in mstatus
    pub fn global_interrupts_enabled(&self) -> bool {
        (self.mstatus & (1 << 3)) != 0
    }
}

/// Check if an address is within valid RAM bounds
///
/// # Arguments
/// * `addr` - The address to validate
///
/// # Returns
/// `true` if the address is within the valid RAM range
pub fn is_valid_ram_address(addr: usize) -> bool {
    addr >= memory_map::RAM_START && addr < memory_map::RAM_END
}

/// Check if an address is properly aligned
///
/// # Arguments
/// * `addr` - The address to check
/// * `alignment` - Required alignment in bytes
///
/// # Returns
/// `true` if the address is properly aligned
pub fn is_aligned(addr: usize, alignment: usize) -> bool {
    addr % alignment == 0
}

/// Get the hardware thread (hart) identifier
///
/// # Returns
/// The unique identifier for this hardware thread
pub fn get_hart_id() -> u64 {
    let mut val: u64;
    unsafe {
        core::arch::asm!("csrr {}, mhartid", out(reg) val);
    }
    val
}

/// Get a string describing the ISA implementation
///
/// # Returns
/// A static string describing the RISC-V ISA features
///
/// # Note
/// This is a simplified version. A complete implementation would
/// read the `misa` CSR to determine actual ISA features.
pub fn get_isa_string() -> &'static str {
    "rv64imac" // Basic RISC-V 64-bit ISA with integer, multiply, atomic, compressed
}

/// Print detailed hardware information
///
/// Displays comprehensive information about the RISC-V hardware,
/// including hart ID, ISA features, and current processor state.
/// Useful for system debugging and hardware introspection.
pub fn print_hardware_info() {
    crate::println!("=== RISC-V Hardware Information ===");
    crate::println_number!("Hart ID: ", get_hart_id());
    crate::print!("ISA: ");
    crate::println!(get_isa_string());

    let context = RiscvContext::capture();
    crate::println_hex!("MSTATUS: ", context.mstatus);
    crate::println_hex!("MTVEC:   ", context.mtvec);

    if context.global_interrupts_enabled() {
        crate::println!("Global interrupts: ENABLED");
    } else {
        crate::println!("Global interrupts: DISABLED");
    }
}
