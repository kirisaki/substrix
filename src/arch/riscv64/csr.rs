// src/arch/riscv64/csr.rs
//! RISC-V Control and Status Register Operations
//!
//! This module provides both low-level CSR access functions and a higher-level
//! hardware abstraction layer interface for RISC-V control and status registers.
//! It includes both legacy compatibility functions and new HAL-compliant interfaces.

use super::RiscvError;
use crate::arch::{ControlStatusRegister, Register};

/// CSR register identifiers
///
/// Enumeration of the control and status registers that can be accessed
/// through the generic CSR interface.
#[derive(Debug, Clone, Copy)]
pub enum CsrId {
    /// Machine Status register - controls global interrupt enable and privilege
    MStatus,
    /// Machine Interrupt Enable register - controls individual interrupt enables
    MIE,
    /// Machine Interrupt Pending register - shows pending interrupts
    MIP,
    /// Machine Trap Vector Base Address register - trap handler address
    MTvec,
    /// Machine Cause register - shows cause of last trap
    MCause,
    /// Machine Exception Program Counter - return address for traps
    MEpc,
    /// Machine Hart ID register - hardware thread identifier
    MHartId,
}

/// Generic CSR register wrapper
///
/// Provides a type-safe interface to RISC-V control and status registers
/// through the Hardware Abstraction Layer traits.
pub struct Csr {
    id: CsrId,
}

impl Csr {
    /// Create a new CSR wrapper for the specified register
    ///
    /// # Arguments
    /// * `id` - The CSR identifier
    ///
    /// # Returns
    /// A new `Csr` instance for the specified register
    pub const fn new(id: CsrId) -> Self {
        Self { id }
    }
}

impl ControlStatusRegister for Csr {
    /// Read the current value of this CSR
    ///
    /// # Returns
    /// The current register value
    fn read(&self) -> Register {
        match self.id {
            CsrId::MStatus => read_mstatus(),
            CsrId::MIE => read_mie(),
            CsrId::MIP => read_mip(),
            CsrId::MTvec => read_mtvec(),
            CsrId::MCause => read_mcause(),
            CsrId::MEpc => read_mepc(),
            CsrId::MHartId => read_mhartid() as Register,
        }
    }

    /// Write a value to this CSR
    ///
    /// # Arguments
    /// * `value` - The value to write
    ///
    /// # Safety
    /// This function is unsafe because writing to CSRs can affect system state,
    /// interrupt handling, and privilege levels.
    ///
    /// # Note
    /// Read-only registers (MIP, MCause, MHartId) will silently ignore writes.
    unsafe fn write(&self, value: Register) {
        match self.id {
            CsrId::MStatus => write_mstatus(value),
            CsrId::MIE => write_mie(value),
            CsrId::MTvec => write_mtvec(value),
            CsrId::MEpc => write_mepc(value),
            // Read-only registers - no operation performed
            CsrId::MIP | CsrId::MCause | CsrId::MHartId => {
                // Could log an error here in a full implementation
            }
        }
    }
}

/// Static CSR instances for type-safe access
///
/// These constants provide convenient access to CSRs through the HAL interface
/// while maintaining compile-time type safety.

/// Machine Status register instance
pub static MSTATUS: Csr = Csr::new(CsrId::MStatus);

/// Machine Interrupt Enable register instance
pub static MIE: Csr = Csr::new(CsrId::MIE);

/// Machine Interrupt Pending register instance
pub static MIP: Csr = Csr::new(CsrId::MIP);

/// Machine Trap Vector Base Address register instance
pub static MTVEC: Csr = Csr::new(CsrId::MTvec);

/// Machine Cause register instance
pub static MCAUSE: Csr = Csr::new(CsrId::MCause);

/// Machine Exception Program Counter register instance
pub static MEPC: Csr = Csr::new(CsrId::MEpc);

/// Machine Hart ID register instance
pub static MHARTID: Csr = Csr::new(CsrId::MHartId);

// Legacy compatibility functions
// These functions maintain compatibility with existing code while we transition to HAL

/// Write to Machine Trap Vector Base Address register
///
/// # Arguments
/// * `addr` - The address of the trap handler
///
/// # Safety
/// This function is unsafe because setting the trap vector affects
/// exception and interrupt handling for the entire system.
pub unsafe fn write_mtvec(addr: usize) {
    core::arch::asm!("csrw mtvec, {}", in(reg) addr);
}

/// Read Machine Trap Vector Base Address register
///
/// # Returns
/// The current trap vector base address
pub fn read_mtvec() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mtvec", out(reg) val);
    }
    val
}

/// Read Machine Cause register
///
/// # Returns
/// The cause of the most recent trap (exception or interrupt)
pub fn read_mcause() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mcause", out(reg) val);
    }
    val
}

/// Read Machine Exception Program Counter
///
/// # Returns
/// The program counter value at the time of the most recent trap
pub fn read_mepc() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mepc", out(reg) val);
    }
    val
}

/// Write to Machine Exception Program Counter
///
/// # Arguments
/// * `addr` - The address to return to when exiting a trap
///
/// # Safety
/// This function is unsafe because modifying the exception PC affects
/// control flow when returning from trap handlers.
pub unsafe fn write_mepc(addr: usize) {
    core::arch::asm!("csrw mepc, {}", in(reg) addr);
}

/// Read Machine Status register
///
/// # Returns
/// The current machine status, including interrupt enable state
pub fn read_mstatus() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mstatus", out(reg) val);
    }
    val
}

/// Write to Machine Status register
///
/// # Arguments
/// * `val` - The new status value
///
/// # Safety
/// This function is unsafe because the machine status register controls
/// interrupt enables, privilege levels, and other critical system state.
pub unsafe fn write_mstatus(val: usize) {
    core::arch::asm!("csrw mstatus, {}", in(reg) val);
}

/// Read Machine Interrupt Enable register
///
/// # Returns
/// A bitmask indicating which interrupts are enabled
pub fn read_mie() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mie", out(reg) val);
    }
    val
}

/// Write to Machine Interrupt Enable register
///
/// # Arguments
/// * `val` - Bitmask of interrupts to enable
///
/// # Safety
/// This function is unsafe because enabling/disabling interrupts affects
/// system responsiveness and real-time behavior.
pub unsafe fn write_mie(val: usize) {
    core::arch::asm!("csrw mie, {}", in(reg) val);
}

/// Read Machine Interrupt Pending register
///
/// # Returns
/// A bitmask indicating which interrupts are currently pending
pub fn read_mip() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mip", out(reg) val);
    }
    val
}

/// Read Machine Hart ID register
///
/// # Returns
/// The unique identifier for this hardware thread
pub fn read_mhartid() -> u64 {
    let mut val: u64;
    unsafe {
        core::arch::asm!("csrr {}, mhartid", out(reg) val);
    }
    val
}

/// RISC-V CSR bit field constants
///
/// This module contains bit field definitions for various RISC-V CSRs,
/// making it easier to manipulate specific bits without magic numbers.
pub mod bits {
    // Machine Status register bit fields

    /// Global interrupt enable bit in mstatus
    pub const MSTATUS_MIE: usize = 1 << 3;

    /// Previous interrupt enable bit in mstatus
    pub const MSTATUS_MPIE: usize = 1 << 7;

    /// Previous privilege mode field mask in mstatus
    pub const MSTATUS_MPP_MASK: usize = 3 << 11;

    // Machine Interrupt Enable register bit fields

    /// Machine software interrupt enable bit
    pub const MIE_MSIE: usize = 1 << 3;

    /// Machine timer interrupt enable bit
    pub const MIE_MTIE: usize = 1 << 7;

    /// Machine external interrupt enable bit
    pub const MIE_MEIE: usize = 1 << 11;

    // Machine Interrupt Pending register bit fields

    /// Machine software interrupt pending bit
    pub const MIP_MSIP: usize = 1 << 3;

    /// Machine timer interrupt pending bit
    pub const MIP_MTIP: usize = 1 << 7;

    /// Machine external interrupt pending bit
    pub const MIP_MEIP: usize = 1 << 11;

    // Machine Cause register bit fields

    /// Interrupt bit in mcause (bit 63)
    pub const MCAUSE_INTERRUPT_BIT: usize = 1 << 63;

    /// Exception code mask in mcause
    pub const MCAUSE_EXCEPTION_MASK: usize = 0x7FFFFFFFFFFFFFFF;

    // Exception codes for mcause register

    /// Instruction address misaligned exception
    pub const EXCEPTION_INSTR_MISALIGNED: usize = 0;

    /// Instruction access fault exception
    pub const EXCEPTION_INSTR_ACCESS_FAULT: usize = 1;

    /// Illegal instruction exception
    pub const EXCEPTION_ILLEGAL_INSTR: usize = 2;

    /// Breakpoint exception
    pub const EXCEPTION_BREAKPOINT: usize = 3;

    /// Load address misaligned exception
    pub const EXCEPTION_LOAD_MISALIGNED: usize = 4;

    /// Load access fault exception
    pub const EXCEPTION_LOAD_ACCESS_FAULT: usize = 5;

    /// Store/AMO address misaligned exception
    pub const EXCEPTION_STORE_MISALIGNED: usize = 6;

    /// Store/AMO access fault exception
    pub const EXCEPTION_STORE_ACCESS_FAULT: usize = 7;

    /// Environment call from User mode
    pub const EXCEPTION_ECALL_UMODE: usize = 8;

    /// Environment call from Supervisor mode
    pub const EXCEPTION_ECALL_SMODE: usize = 9;

    /// Environment call from Machine mode
    pub const EXCEPTION_ECALL_MMODE: usize = 11;

    // Interrupt codes for mcause register

    /// Machine software interrupt
    pub const INTERRUPT_SW_MACHINE: usize = 3;

    /// Machine timer interrupt
    pub const INTERRUPT_TIMER_MACHINE: usize = 7;

    /// Machine external interrupt
    pub const INTERRUPT_EXT_MACHINE: usize = 11;
}

// High-level interrupt control functions

/// Enable machine timer interrupts
///
/// Sets the MTIE bit in the MIE register to enable timer interrupts.
///
/// # Returns
/// `Ok(())` on success, `Err(RiscvError::HardwareFault)` if verification fails
///
/// # Safety
/// This function is unsafe because enabling timer interrupts affects
/// system scheduling and real-time behavior.
pub unsafe fn enable_machine_timer_interrupt() -> Result<(), RiscvError> {
    let mut mie = read_mie();
    mie |= bits::MIE_MTIE;
    write_mie(mie);

    // Verify the write succeeded
    let readback = read_mie();
    if (readback & bits::MIE_MTIE) != 0 {
        Ok(())
    } else {
        Err(RiscvError::HardwareFault)
    }
}

/// Enable machine external interrupts
///
/// Sets the MEIE bit in the MIE register to enable external interrupts.
///
/// # Returns
/// `Ok(())` on success, `Err(RiscvError::HardwareFault)` if verification fails
///
/// # Safety
/// This function is unsafe because enabling external interrupts affects
/// how the system responds to hardware events.
pub unsafe fn enable_machine_external_interrupt() -> Result<(), RiscvError> {
    let mut mie = read_mie();
    mie |= bits::MIE_MEIE;
    write_mie(mie);

    let readback = read_mie();
    if (readback & bits::MIE_MEIE) != 0 {
        Ok(())
    } else {
        Err(RiscvError::HardwareFault)
    }
}

/// Enable machine software interrupts
///
/// Sets the MSIE bit in the MIE register to enable software interrupts.
///
/// # Returns
/// `Ok(())` on success, `Err(RiscvError::HardwareFault)` if verification fails
///
/// # Safety
/// This function is unsafe because enabling software interrupts affects
/// inter-processor communication and task scheduling.
pub unsafe fn enable_machine_software_interrupt() -> Result<(), RiscvError> {
    let mut mie = read_mie();
    mie |= bits::MIE_MSIE;
    write_mie(mie);

    let readback = read_mie();
    if (readback & bits::MIE_MSIE) != 0 {
        Ok(())
    } else {
        Err(RiscvError::HardwareFault)
    }
}

/// Enable global interrupts
///
/// Sets the MIE bit in the mstatus register to enable interrupt handling.
///
/// # Returns
/// `Ok(())` on success, `Err(RiscvError::HardwareFault)` if verification fails
///
/// # Safety
/// This function is unsafe because enabling global interrupts affects
/// system concurrency and timing behavior.
pub unsafe fn enable_global_interrupts() -> Result<(), RiscvError> {
    let mut mstatus = read_mstatus();
    mstatus |= bits::MSTATUS_MIE;
    write_mstatus(mstatus);

    let readback = read_mstatus();
    if (readback & bits::MSTATUS_MIE) != 0 {
        Ok(())
    } else {
        Err(RiscvError::HardwareFault)
    }
}

/// Disable global interrupts
///
/// Clears the MIE bit in the mstatus register to disable interrupt handling.
///
/// # Returns
/// `Ok(())` on success, `Err(RiscvError::HardwareFault)` if verification fails
///
/// # Safety
/// This function is unsafe because disabling global interrupts can affect
/// system responsiveness and real-time guarantees.
pub unsafe fn disable_global_interrupts() -> Result<(), RiscvError> {
    let mut mstatus = read_mstatus();
    mstatus &= !bits::MSTATUS_MIE;
    write_mstatus(mstatus);

    let readback = read_mstatus();
    if (readback & bits::MSTATUS_MIE) == 0 {
        Ok(())
    } else {
        Err(RiscvError::HardwareFault)
    }
}

/// Check if global interrupts are currently enabled
///
/// # Returns
/// `true` if global interrupts are enabled, `false` otherwise
pub fn interrupts_enabled() -> bool {
    let mstatus = read_mstatus();
    (mstatus & bits::MSTATUS_MIE) != 0
}

/// Interrupt types for checking enable status
#[derive(Debug, Clone, Copy)]
pub enum InterruptType {
    /// Software interrupts (MSIE)
    Software,
    /// Timer interrupts (MTIE)
    Timer,
    /// External interrupts (MEIE)
    External,
}

/// Check if a specific interrupt type is enabled
///
/// # Arguments
/// * `interrupt_type` - The type of interrupt to check
///
/// # Returns
/// `true` if the specified interrupt type is enabled
pub fn is_interrupt_enabled(interrupt_type: InterruptType) -> bool {
    let mie = read_mie();
    match interrupt_type {
        InterruptType::Software => (mie & bits::MIE_MSIE) != 0,
        InterruptType::Timer => (mie & bits::MIE_MTIE) != 0,
        InterruptType::External => (mie & bits::MIE_MEIE) != 0,
    }
}

// Legacy compatibility aliases
// These maintain compatibility with existing code during the transition period

/// Legacy alias for enable_machine_timer_interrupt
pub use enable_machine_timer_interrupt as enable_machine_timer_interrupt_legacy;

/// Legacy alias for enable_machine_external_interrupt
pub use enable_machine_external_interrupt as enable_machine_external_interrupt_legacy;

/// Legacy alias for enable_machine_software_interrupt
pub use enable_machine_software_interrupt as enable_machine_software_interrupt_legacy;

/// Legacy alias for enable_global_interrupts
pub use enable_global_interrupts as enable_global_interrupts_legacy;

/// Legacy alias for disable_global_interrupts
pub use disable_global_interrupts as disable_global_interrupts_legacy;
