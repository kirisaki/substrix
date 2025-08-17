// src/arch/riscv64/timer.rs
//! RISC-V Timer Implementation (Unified)
//!
//! This module provides the complete RISC-V timer implementation using the
//! Core-Local Interruptor (CLINT) for QEMU virt machine. All timer functionality
//! is consolidated here for clean architecture.

use super::{memory_map, RiscvError};
use crate::arch::Timer;
use crate::console::{hex, num, str};
use crate::UART0;

/// RISC-V timer frequency for QEMU virt machine (10 MHz)
pub const TIMER_FREQ: u64 = 10_000_000;

/// Timer duration type (64-bit tick count)
pub type TimerDuration = u64;

/// RISC-V CLINT Timer implementation
///
/// This structure provides access to the RISC-V Core-Local Interruptor
/// timer functionality, including MTIME and MTIMECMP registers.
pub struct ClintTimer {
    /// Base address of MTIME register
    mtime_addr: *const u64,

    /// Base address of MTIMECMP register
    mtimecmp_addr: *mut u64,

    /// Timer frequency in Hz
    frequency: u64,
}

// Safety: In a bare-metal single-core environment, sharing raw pointers
// between threads is not a concern as there are no threads. The hardware
// registers are memory-mapped and safe to access from the single execution context.
unsafe impl Sync for ClintTimer {}

impl ClintTimer {
    /// Create a new CLINT timer instance
    ///
    /// # Returns
    /// A new `ClintTimer` instance configured for QEMU virt machine
    pub const fn new() -> Self {
        Self {
            mtime_addr: memory_map::MTIME_ADDR as *const u64,
            mtimecmp_addr: memory_map::MTIMECMP_BASE as *mut u64,
            frequency: TIMER_FREQ,
        }
    }

    /// Read the MTIME register directly
    ///
    /// # Returns
    /// Current value of the MTIME register
    pub fn read_mtime(&self) -> u64 {
        unsafe { core::ptr::read_volatile(self.mtime_addr) }
    }

    /// Write to the MTIMECMP register directly
    ///
    /// # Arguments
    /// * `value` - The value to write to MTIMECMP
    ///
    /// # Safety
    /// This function is unsafe because writing to MTIMECMP affects
    /// timer interrupt generation.
    pub unsafe fn write_mtimecmp(&self, value: u64) {
        core::ptr::write_volatile(self.mtimecmp_addr, value);
    }

    /// Read the MTIMECMP register directly
    ///
    /// # Returns
    /// Current value of the MTIMECMP register
    pub fn read_mtimecmp(&self) -> u64 {
        unsafe { core::ptr::read_volatile(self.mtimecmp_addr) }
    }

    /// Check if the timer is properly accessible
    ///
    /// # Returns
    /// `true` if the timer hardware is accessible and functional
    pub fn is_accessible(&self) -> bool {
        let mtime1 = self.read_mtime();

        // Brief delay to allow timer to advance
        for _ in 0..1000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }

        let mtime2 = self.read_mtime();

        // Timer should advance (or at least not go backwards)
        mtime2 >= mtime1 && mtime1 > 0
    }

    /// Initialize the timer to a safe state
    ///
    /// Sets MTIMECMP to a very far future value to prevent
    /// immediate timer interrupts during initialization.
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(RiscvError)` on failure
    pub fn initialize(&self) -> Result<(), RiscvError> {
        crate::println!("=== TIMER SYSTEM INITIALIZATION ===");

        // Check if timer is accessible
        crate::println!("Testing timer hardware access...");
        if !self.is_accessible() {
            crate::println!("✗ Timer hardware not accessible");
            return Err(RiscvError::HardwareFault);
        }
        crate::println!("✓ Timer hardware accessible");

        // Set MTIMECMP to far future to prevent immediate interrupts
        let current_time = self.read_mtime();
        let safe_future = current_time + (self.frequency * 3600); // 1 hour from now

        crate::println!("Setting timer to safe state...");
        unsafe {
            self.write_mtimecmp(safe_future);
        }

        // Verify the write succeeded
        let readback = self.read_mtimecmp();
        if readback == safe_future {
            crate::println!("✓ Timer initialized to safe state");
            crate::println!("Current MTIME: {}", num(current_time));
            crate::println!("MTIMECMP set to: {}", num(safe_future));
            Ok(())
        } else {
            crate::println!("✗ Timer initialization verification failed");
            Err(RiscvError::HardwareFault)
        }
    }
}

impl Timer for ClintTimer {
    type Error = RiscvError;
    type Duration = TimerDuration;

    /// Get the current time from MTIME register
    fn now(&self) -> Self::Duration {
        self.read_mtime()
    }

    /// Set timer alarm for absolute time
    unsafe fn set_alarm(&self, when: Self::Duration) -> Result<(), Self::Error> {
        self.write_mtimecmp(when);

        // Verify the write succeeded
        let readback = self.read_mtimecmp();
        if readback == when {
            TIMER_STATS.record_alarm_set();
            Ok(())
        } else {
            TIMER_STATS.record_error();
            Err(RiscvError::HardwareFault)
        }
    }

    /// Stop the timer by setting MTIMECMP to maximum value
    unsafe fn stop(&self) -> Result<(), Self::Error> {
        self.write_mtimecmp(u64::MAX);

        // Verify the write succeeded
        let readback = self.read_mtimecmp();
        if readback == u64::MAX {
            Ok(())
        } else {
            TIMER_STATS.record_error();
            Err(RiscvError::HardwareFault)
        }
    }

    /// Get timer frequency in Hz
    fn frequency(&self) -> u64 {
        self.frequency
    }

    /// Convert timer ticks to milliseconds
    fn ticks_to_ms(&self, ticks: Self::Duration) -> u64 {
        ticks / (self.frequency / 1000)
    }

    /// Convert milliseconds to timer ticks
    fn ms_to_ticks(&self, ms: u64) -> Self::Duration {
        ms * (self.frequency / 1000)
    }
}

/// Global CLINT timer instance
pub static CLINT_TIMER: ClintTimer = ClintTimer::new();

/// Timer statistics tracking
#[derive(Debug, Clone, Copy)]
pub struct TimerStats {
    /// Number of timer interrupts handled
    pub interrupts: u64,

    /// Number of times timer alarm was set
    pub alarms_set: u64,

    /// Number of timer-related errors
    pub errors: u64,

    /// Total ticks elapsed since initialization
    pub total_ticks: u64,
}

impl TimerStats {
    /// Create a new empty statistics structure
    const fn new() -> Self {
        Self {
            interrupts: 0,
            alarms_set: 0,
            errors: 0,
            total_ticks: 0,
        }
    }

    /// Record a timer interrupt
    fn record_interrupt(&mut self) {
        self.interrupts = self.interrupts.wrapping_add(1);
    }

    /// Record an alarm being set
    fn record_alarm_set(&mut self) {
        self.alarms_set = self.alarms_set.wrapping_add(1);
    }

    /// Record a timer error
    fn record_error(&mut self) {
        self.errors = self.errors.wrapping_add(1);
    }

    /// Update total ticks
    fn update_ticks(&mut self, current_ticks: u64) {
        self.total_ticks = current_ticks;
    }
}

/// Global timer statistics
static mut TIMER_STATS: TimerStats = TimerStats::new();

/// Get current timer statistics
pub fn get_timer_stats() -> TimerStats {
    unsafe { TIMER_STATS }
}

/// Handle timer interrupt (called from trap handler)
///
/// This function processes timer interrupts and sets up the next interrupt.
pub fn handle_timer_interrupt() {
    unsafe {
        TIMER_STATS.record_interrupt();
    }

    // Set next timer interrupt (10 seconds interval)
    let current_time = CLINT_TIMER.now();
    let next_interrupt = current_time + (CLINT_TIMER.frequency() * 10);

    unsafe {
        if let Err(_) = CLINT_TIMER.set_alarm(next_interrupt) {
            TIMER_STATS.record_error();
        }
    }

    // Simple output for interrupt indication
    unsafe {
        let interrupts = TIMER_STATS.interrupts;

        // Output tick marker
        core::ptr::write_volatile(UART0, b'T');
        core::ptr::write_volatile(UART0, b'K');

        // Output interrupt count in hex (last digit)
        let tick_low = interrupts & 0xF;
        let hex_char = if tick_low < 10 {
            b'0' + tick_low as u8
        } else {
            b'a' + (tick_low - 10) as u8
        };
        core::ptr::write_volatile(UART0, hex_char);
        core::ptr::write_volatile(UART0, b'\n');
    }
}

/// Timer utility functions
pub mod utils {
    use super::*;

    /// Get current time in milliseconds since system start
    pub fn current_time_ms() -> u64 {
        let current_ticks = CLINT_TIMER.now();
        CLINT_TIMER.ticks_to_ms(current_ticks)
    }

    /// Delay for specified number of milliseconds
    ///
    /// This function performs a busy-wait delay using the timer.
    pub fn delay_ms(ms: u64) {
        let start_time = CLINT_TIMER.now();
        let delay_ticks = CLINT_TIMER.ms_to_ticks(ms);
        let target_time = start_time + delay_ticks;

        while CLINT_TIMER.now() < target_time {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    /// Check if a timeout has expired
    pub fn is_timeout(start_time: u64, timeout_ms: u64) -> bool {
        let current_time = CLINT_TIMER.now();
        let timeout_ticks = CLINT_TIMER.ms_to_ticks(timeout_ms);

        current_time >= start_time + timeout_ticks
    }

    /// Measure execution time of a closure
    pub fn measure_time<F, R>(f: F) -> (R, u64)
    where
        F: FnOnce() -> R,
    {
        let start_time = CLINT_TIMER.now();
        let result = f();
        let end_time = CLINT_TIMER.now();

        let elapsed_ticks = end_time - start_time;
        let elapsed_ms = CLINT_TIMER.ticks_to_ms(elapsed_ticks);

        (result, elapsed_ms)
    }
}

/// Timer system management functions
pub mod system {
    use super::*;

    /// Initialize the complete timer system
    pub fn init() -> Result<(), RiscvError> {
        CLINT_TIMER.initialize()
    }

    /// Display timer system information
    pub fn show_info() {
        crate::println!("=== TIMER SYSTEM INFORMATION ===");

        // Hardware information
        crate::println!("Hardware:");
        crate::println!("  MTIME address: {}", hex(memory_map::MTIME_ADDR));
        crate::println!("  MTIMECMP address: {}", hex(memory_map::MTIMECMP_BASE));
        crate::println!("  Frequency: {} Hz", num(CLINT_TIMER.frequency()));

        // Current state
        let current_time = CLINT_TIMER.now();
        let current_mtimecmp = CLINT_TIMER.read_mtimecmp();
        let current_ms = utils::current_time_ms();

        crate::println!("Current state:");
        crate::println!("  MTIME: {}", num(current_time));
        crate::println!("  MTIMECMP: {}", num(current_mtimecmp));
        crate::println!("  Time (ms): {}", num(current_ms));

        // Next interrupt timing
        if current_mtimecmp > current_time {
            let remaining = current_mtimecmp - current_time;
            let seconds = remaining / CLINT_TIMER.frequency();
            crate::println!("  Next interrupt in: {} seconds", num(seconds));
        } else {
            crate::println!("  Next interrupt: immediate or past");
        }

        // Statistics
        unsafe {
            TIMER_STATS.update_ticks(current_time);
        }
        let stats = get_timer_stats();

        crate::println!("Statistics:");
        crate::println!("  Interrupts: {}", num(stats.interrupts));
        crate::println!("  Alarms set: {}", num(stats.alarms_set));
        crate::println!("  Errors: {}", num(stats.errors));

        if stats.errors > 0 && stats.alarms_set > 0 {
            let error_rate = (stats.errors * 100) / stats.alarms_set;
            crate::println!("  Error rate: {}%", num(error_rate));
        }
    }

    /// Prepare timer interrupts with specified interval
    pub fn prepare_interrupts(interval_ms: u64) -> Result<(), RiscvError> {
        crate::println!("=== PREPARING TIMER INTERRUPTS ===");

        let current_time = CLINT_TIMER.now();
        let interval_ticks = CLINT_TIMER.ms_to_ticks(interval_ms);
        let first_interrupt = current_time + interval_ticks;

        crate::println!("Setting timer interrupt:");
        crate::println!("  Current time: {}", num(current_time));
        crate::println!("  Interval: {} ms", num(interval_ms));
        crate::println!("  Target time: {}", num(first_interrupt));

        unsafe {
            match CLINT_TIMER.set_alarm(first_interrupt) {
                Ok(()) => {
                    crate::println!("✓ Timer interrupt prepared successfully");
                    Ok(())
                }
                Err(e) => {
                    crate::println!("✗ Failed to prepare timer interrupt");
                    Err(e)
                }
            }
        }
    }

    /// Test timer delay functionality
    pub fn test_delay() {
        crate::println!("=== TIMER DELAY TEST ===");

        let delay_ms = 1000; // 1 second
        crate::println!("Testing {} ms delay...", num(delay_ms));

        let (_, elapsed_ms) = utils::measure_time(|| {
            utils::delay_ms(delay_ms);
        });

        crate::println!("Requested delay: {} ms", num(delay_ms));
        crate::println!("Actual delay: {} ms", num(elapsed_ms));

        let accuracy = if elapsed_ms > 0 {
            ((delay_ms as i64 - elapsed_ms as i64).abs() as u64 * 100) / delay_ms
        } else {
            100
        };

        crate::println!("Accuracy: {}% deviation", num(accuracy));

        if accuracy <= 10 {
            // Within 10%
            crate::println!("✓ Delay accuracy acceptable");
        } else {
            crate::println!("⚠ Delay accuracy outside acceptable range");
        }
    }

    /// Test timer interrupt functionality
    pub fn test_short_interrupt(delay_ms: u64) -> Result<(), RiscvError> {
        crate::println!("=== SHORT TIMER INTERRUPT TEST ===");

        let current_time = CLINT_TIMER.now();
        let test_interval = CLINT_TIMER.ms_to_ticks(delay_ms);
        let test_target = current_time + test_interval;

        crate::println!("Setting {} ms timer interrupt...", num(delay_ms));
        crate::println!("  Current time: {}", num(current_time));
        crate::println!("  Target time: {}", num(test_target));

        unsafe {
            CLINT_TIMER.set_alarm(test_target)?;
        }

        // Wait for the target time with timeout
        let timeout_ms = delay_ms + 5000; // Extra 5 seconds
        let start_wait = utils::current_time_ms();

        crate::println!("Waiting for timer interrupt...");
        while utils::current_time_ms() - start_wait < timeout_ms {
            let now = CLINT_TIMER.now();

            if now >= test_target {
                crate::println!("✓ Timer target reached");

                // Reset to safe state
                let safe_future = now + CLINT_TIMER.ms_to_ticks(60000); // 1 minute
                unsafe {
                    CLINT_TIMER.set_alarm(safe_future)?;
                }

                return Ok(());
            }

            // Brief CPU relief
            unsafe {
                core::arch::asm!("nop");
            }
        }

        // Timeout - reset to safe state
        let safe_future = CLINT_TIMER.now() + CLINT_TIMER.ms_to_ticks(60000);
        unsafe {
            CLINT_TIMER.set_alarm(safe_future)?;
        }

        Err(RiscvError::HardwareFault)
    }
}

/// Timer testing functions
pub mod test {
    use super::*;

    /// Test basic timer functionality
    pub fn basic() {
        crate::println!("=== BASIC TIMER TEST ===");

        // Test timer accessibility
        if CLINT_TIMER.is_accessible() {
            crate::println!("✓ Timer hardware accessible");
        } else {
            crate::println!("✗ Timer hardware not accessible");
            return;
        }

        // Test current time reading
        let time1 = CLINT_TIMER.now();
        utils::delay_ms(1);
        let time2 = CLINT_TIMER.now();

        crate::println!("Time progression test:");
        crate::println!("  Start: {}", num(time1));
        crate::println!("  End:   {}", num(time2));

        if time2 > time1 {
            crate::println!("✓ Timer advancing correctly");
        } else {
            crate::println!("✗ Timer not advancing");
        }

        // Test frequency
        let freq = CLINT_TIMER.frequency();
        crate::println!("Timer frequency: {} Hz", num(freq));

        // Test time conversion
        let test_ms = 1000u64; // 1 second
        let ticks = CLINT_TIMER.ms_to_ticks(test_ms);
        let back_to_ms = CLINT_TIMER.ticks_to_ms(ticks);

        crate::println!(
            "Conversion test: {} ms = {} ticks = {} ms",
            num(test_ms),
            num(ticks),
            num(back_to_ms)
        );

        if back_to_ms == test_ms {
            crate::println!("✓ Time conversion accurate");
        } else {
            crate::println!("⚠ Time conversion has rounding error");
        }

        crate::println!("✓ Basic timer test completed");
    }

    /// Test timer alarm functionality
    pub fn alarm() {
        crate::println!("=== TIMER ALARM TEST ===");

        let current_time = CLINT_TIMER.now();
        let alarm_delay = CLINT_TIMER.ms_to_ticks(100); // 100ms delay
        let alarm_time = current_time + alarm_delay;

        crate::println!("Setting alarm:");
        crate::println!("  Current time: {}", num(current_time));
        crate::println!("  Alarm time:   {}", num(alarm_time));
        crate::println!("  Delay:        {} ms", num(100));

        // Set the alarm
        unsafe {
            match CLINT_TIMER.set_alarm(alarm_time) {
                Ok(()) => {
                    crate::println!("✓ Alarm set successfully");
                }
                Err(_) => {
                    crate::println!("✗ Failed to set alarm");
                    return;
                }
            }
        }

        // Wait for alarm to trigger (or timeout)
        let max_wait = current_time + alarm_delay + CLINT_TIMER.ms_to_ticks(200); // Extra 200ms

        while CLINT_TIMER.now() < max_wait {
            if CLINT_TIMER.now() >= alarm_time {
                crate::println!("✓ Alarm time reached");
                break;
            }

            unsafe {
                core::arch::asm!("nop");
            }
        }

        // Reset timer to safe state
        let safe_future = CLINT_TIMER.now() + CLINT_TIMER.ms_to_ticks(60000); // 1 minute
        unsafe {
            let _ = CLINT_TIMER.set_alarm(safe_future);
        }

        crate::println!("✓ Timer alarm test completed");
    }

    /// Comprehensive timer test suite
    pub fn comprehensive() {
        crate::println!("=== COMPREHENSIVE TIMER TEST SUITE ===");

        basic();
        alarm();
        system::test_delay();

        // Performance test
        performance();

        crate::println!("✓ Comprehensive timer test completed");
    }

    /// Timer performance benchmark
    pub fn performance() {
        crate::println!("=== TIMER PERFORMANCE TEST ===");

        const ITERATIONS: usize = 10000;

        let (_, duration_ms) = utils::measure_time(|| {
            for _ in 0..ITERATIONS {
                let _ = CLINT_TIMER.now();
            }
        });

        crate::println!(
            "Performance benchmark ({} iterations):",
            num(ITERATIONS as u64)
        );
        crate::println!("  Total time: {} ms", num(duration_ms));

        if duration_ms > 0 {
            let ns_per_op = (duration_ms * 1_000_000) / ITERATIONS as u64;
            crate::println!("  Time per operation: {} ns", num(ns_per_op));
        }

        crate::println!("✓ Performance test completed");
    }
}
