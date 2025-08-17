#![no_std]
#![no_main]

#[macro_use]
mod console;

mod arch;
mod interrupt;
mod msip_debug;
mod panic;
mod trap;

pub const UART0: *mut u8 = 0x1000_0000 as *mut u8;

use crate::arch::{
    current::timer::{system, test, utils, CLINT_TIMER},
    Timer,
};
use crate::console::{hex, num, str};
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("RISC-V Unikernel with Unified HAL Timer System");

    // Phase 1: Basic system initialization
    println!("\n=== PHASE 1: BASIC TESTS ===");
    basic_tests();

    // Phase 2: CSR state analysis
    println!("\n=== PHASE 2: CSR STATE ANALYSIS ===");
    analyze_csr_state();

    // Phase 2.5: HAL System Test
    println!("\n=== PHASE 2.5: HAL SYSTEM TEST ===");
    test_hal_system();

    // Phase 3: Safe trap initialization
    println!("\n=== PHASE 3: SAFE TRAP INITIALIZATION ===");
    initialize_trap_system();

    // Phase 4: Safe ecall test
    println!("\n=== PHASE 4: SAFE ECALL TEST ===");
    test_ecall_functionality();

    // Phase 5: System stability check
    println!("\n=== PHASE 5: SYSTEM STABILITY CHECK ===");
    test_system_stability();

    // Phase 6: MSIP safety verification
    println!("\n=== PHASE 6: MSIP SAFETY VERIFICATION ===");
    test_msip_functionality();

    // Phase 7: Basic MSIP operations
    println!("\n=== PHASE 7: BASIC MSIP OPERATIONS ===");
    test_msip_operations();

    // Phase 8: Software interrupt system
    println!("\n=== PHASE 8: SOFTWARE INTERRUPT SYSTEM ===");
    test_software_interrupt_system();

    // Phase 8.5: Unified timer system
    println!("\n=== PHASE 8.5: UNIFIED TIMER SYSTEM ===");
    test_unified_timer_system();

    // Phase 9: Simple yield test
    println!("\n=== PHASE 9: SIMPLE YIELD TEST ===");
    test_yield_functionality();

    // Phase 10: Timer interrupt system
    println!("\n=== PHASE 10: TIMER INTERRUPT SYSTEM ===");
    test_timer_interrupt_system();

    // Phase 11: Integrated interrupt system
    println!("\n=== PHASE 11: INTEGRATED INTERRUPT SYSTEM ===");
    test_integrated_interrupt_system();

    // Phase 12: Live timer interrupt test
    println!("\n=== PHASE 12: LIVE TIMER INTERRUPT TEST ===");
    test_live_timer_interrupts();

    // Phase 13: Panic system test
    println!("\n=== PHASE 13: PANIC SYSTEM TEST ===");
    test_panic_system();

    // Phase 14: Main system loop
    println!("\n=== PHASE 14: MAIN SYSTEM LOOP ===");
    main_system_loop();
}

/// Basic functionality tests
fn basic_tests() {
    println!("Running basic system tests...");

    // Arithmetic test
    let result = 2 + 2;
    println!("Arithmetic test: 2 + 2 = {}", num(result));
    if result == 4 {
        println!("✓ Arithmetic: PASS");
    } else {
        println!("✗ Arithmetic: FAIL");
    }

    // Memory test
    let mut test_array = [1, 2, 3, 4, 5];
    test_array[2] = 99;
    println!("Memory test: array[2] = {}", num(test_array[2]));
    if test_array[2] == 99 {
        println!("✓ Memory: PASS");
    } else {
        println!("✗ Memory: FAIL");
    }

    println!("✓ Basic tests completed");
}

/// Analyze CSR state
fn analyze_csr_state() {
    println!("Analyzing CSR state...");

    let mhartid = read_mhartid();
    let mstatus = arch::csr::read_mstatus();
    let mie = arch::csr::read_mie();
    let mtvec = arch::csr::read_mtvec();
    let mcause = arch::csr::read_mcause();
    let mepc = arch::csr::read_mepc();

    println!("CSR values:");
    println!("  mhartid: {}", num(mhartid));
    println!("  mstatus: {}", hex(mstatus));
    println!("  mie: {}", hex(mie));
    println!("  mtvec: {}", hex(mtvec));
    println!("  mcause: {}", hex(mcause));
    println!("  mepc: {}", hex(mepc));

    // Analyze mstatus bits
    let mie_bit = (mstatus >> 3) & 1;
    let mpie_bit = (mstatus >> 7) & 1;
    let mpp_bits = (mstatus >> 11) & 3;

    println!("mstatus analysis:");
    println!("  MIE: {}", num(mie_bit as u64));
    println!("  MPIE: {}", num(mpie_bit as u64));
    println!("  MPP: {}", num(mpp_bits as u64));

    println!("✓ CSR state analysis complete");
}

/// Test HAL system
fn test_hal_system() {
    println!("Testing Hardware Abstraction Layer...");

    // Display architecture information
    arch::print_arch_info();

    // Display RISC-V specific information
    arch::current::print_hardware_info();

    // Test CSR abstraction
    test_csr_abstraction();

    println!("✓ HAL system test completed");
}

/// Test CSR abstraction
fn test_csr_abstraction() {
    use arch::ControlStatusRegister;

    println!("Testing CSR abstraction API...");

    // Test new HAL-based CSR access
    let mstatus_val = arch::current::csr::MSTATUS.read();
    let mie_val = arch::current::csr::MIE.read();

    println!("HAL CSR access:");
    println!("  MSTATUS: {}", hex(mstatus_val));
    println!("  MIE: {}", hex(mie_val));

    // Verify compatibility with legacy API
    let mstatus_legacy = arch::csr::read_mstatus();
    let mie_legacy = arch::csr::read_mie();

    if mstatus_val == mstatus_legacy && mie_val == mie_legacy {
        println!("✓ HAL and legacy APIs match");
    } else {
        println!("✗ HAL and legacy APIs mismatch");
    }

    // Test bit field operations
    use arch::current::csr::bits;

    let global_ie = (mstatus_val & bits::MSTATUS_MIE) != 0;
    let timer_ie = (mie_val & bits::MIE_MTIE) != 0;
    let sw_ie = (mie_val & bits::MIE_MSIE) != 0;

    println!("Interrupt enable status:");
    println!(
        "  Global: {}",
        str(if global_ie { "ENABLED" } else { "DISABLED" })
    );
    println!(
        "  Timer: {}",
        str(if timer_ie { "ENABLED" } else { "DISABLED" })
    );
    println!(
        "  Software: {}",
        str(if sw_ie { "ENABLED" } else { "DISABLED" })
    );
}

/// Initialize trap system
fn initialize_trap_system() {
    println!("Initializing trap handler...");

    let mtvec_before = arch::csr::read_mtvec();
    println!("mtvec before init: {}", hex(mtvec_before));

    trap::init_trap();

    let mtvec_after = arch::csr::read_mtvec();
    println!("mtvec after init: {}", hex(mtvec_after));

    if mtvec_after != 0 && mtvec_after != mtvec_before {
        println!("✓ Trap handler successfully initialized");
    } else {
        println!("✗ Trap handler initialization failed");
        panic!("Trap init failed");
    }
}

/// Test ecall functionality
fn test_ecall_functionality() {
    println!("Testing ecall (this should trigger trap)...");

    let mcause_before = arch::csr::read_mcause();
    let mepc_before = arch::csr::read_mepc();
    println!(
        "Before ecall - mcause: {}, mepc: {}",
        hex(mcause_before),
        hex(mepc_before)
    );

    trap::test_ecall_safe();

    let mcause_after = arch::csr::read_mcause();
    let mepc_after = arch::csr::read_mepc();
    println!(
        "After ecall - mcause: {}, mepc: {}",
        hex(mcause_after),
        hex(mepc_after)
    );

    println!("✓ Ecall test completed successfully");
}

/// Test system stability
fn test_system_stability() {
    println!("Running stability test with trap handler active...");

    let mut counter = 0u64;
    let stability_test_limit = 30000000;

    println!("Running short stability test...");
    while counter < stability_test_limit {
        counter = counter.wrapping_add(1);

        if counter % 10000000 == 0 {
            println!("Stability test: {}", num(counter));
        }

        unsafe {
            core::arch::asm!("nop");
        }
    }

    println!("✓ Short stability test passed");
}

/// Test MSIP functionality
fn test_msip_functionality() {
    println!("Testing MSIP with active trap handler...");
    msip_debug::comprehensive_clint_test();
}

/// Test MSIP operations
fn test_msip_operations() {
    println!("Testing safe MSIP operations...");
    match msip_debug::safe_msip_read() {
        Ok(val) => {
            println!("Safe MSIP read successful: {}", num(val as u64));
            msip_debug::basic_msip_test();
        }
        Err(e) => {
            println!("MSIP read failed: {}", str(e));
            println!("Skipping MSIP operations");
        }
    }
}

/// Test software interrupt system
fn test_software_interrupt_system() {
    println!("Initializing software interrupt system...");
    interrupt::init_software_interrupt();

    println!("Testing basic MSIP operations after init...");
    match interrupt::test_basic_msip_operations_simple() {
        Ok(()) => println!("✓ Basic MSIP operations work"),
        Err(e) => println!("✗ Basic MSIP operations failed: {}", str(e)),
    }
}

/// Test unified timer system
fn test_unified_timer_system() {
    println!("Testing unified HAL timer system...");

    // Initialize timer system
    println!("Initializing timer system...");
    match system::init() {
        Ok(()) => println!("✓ Timer system initialized"),
        Err(_) => {
            println!("✗ Timer system initialization failed");
            return;
        }
    }

    // Display timer information
    system::show_info();

    // Test basic timer functionality
    println!("Testing basic timer functionality...");
    test::basic();

    // Test timer utilities
    println!("Testing timer utilities...");
    test_timer_utilities();

    // Test timer performance
    println!("Testing timer performance...");
    test::performance();

    println!("✓ Unified timer system test completed");
}

/// Test timer utilities
fn test_timer_utilities() {
    println!("Testing timer utility functions...");

    // Test current time
    let current_ms = utils::current_time_ms();
    println!("Current time: {} ms", num(current_ms));

    // Test delay functionality
    println!("Testing 100ms delay...");
    let (_, actual_delay) = utils::measure_time(|| {
        utils::delay_ms(100);
    });
    println!("Requested: 100ms, Actual: {} ms", num(actual_delay));

    // Test timeout detection
    let start_time = CLINT_TIMER.now();
    utils::delay_ms(50);
    if utils::is_timeout(start_time, 25) {
        println!("✓ Timeout detection working (25ms < 50ms)");
    } else {
        println!("⚠ Timeout detection needs calibration");
    }

    // Test performance measurement
    let (result, exec_time) = utils::measure_time(|| {
        let mut sum = 0u64;
        for i in 0..1000 {
            sum += i;
        }
        sum
    });
    println!(
        "Performance test: result={}, time={}ms",
        num(result),
        num(exec_time)
    );

    println!("✓ Timer utilities test completed");
}

/// Test yield functionality
fn test_yield_functionality() {
    println!("Testing single yield() call...");
    match interrupt::yield_cpu_relaxed() {
        Ok(()) => println!("✓ Single yield successful"),
        Err(e) => println!("✗ Single yield failed: {}", str(e)),
    }
}

/// Test timer interrupt system
fn test_timer_interrupt_system() {
    println!("Setting up timer interrupt system...");

    // Enable timer interrupts in MIE
    println!("Enabling timer interrupts...");
    unsafe {
        arch::csr::enable_machine_timer_interrupt();
    }

    let mie = arch::csr::read_mie();
    if (mie & (1 << 7)) != 0 {
        println!("✓ Machine Timer Interrupt Enable (MTIE) active");
    } else {
        println!("✗ MTIE not enabled");
        return;
    }

    // Prepare timer interrupts
    println!("Preparing timer interrupts...");
    match system::prepare_interrupts(30000) {
        // 30 seconds
        Ok(()) => println!("✓ Timer interrupts prepared"),
        Err(_) => println!("✗ Failed to prepare timer interrupts"),
    }

    println!("✓ Timer interrupt system setup completed");
}

/// Test integrated interrupt system
fn test_integrated_interrupt_system() {
    println!("Testing integrated interrupt system...");

    // Enable global interrupts
    println!("Enabling global interrupts for integrated test...");
    unsafe {
        arch::csr::enable_global_interrupts();
    }

    let mstatus = arch::csr::read_mstatus();
    let mie = arch::csr::read_mie();
    println!("Final interrupt state:");
    println!("  mstatus: {}", hex(mstatus));
    println!("  mie: {}", hex(mie));

    // Check interrupt enable status
    if (mstatus & (1 << 3)) != 0 {
        println!("  ✓ Global interrupts (MIE) enabled");
    }
    if (mie & (1 << 3)) != 0 {
        println!("  ✓ Software interrupts (MSIE) enabled");
    }
    if (mie & (1 << 7)) != 0 {
        println!("  ✓ Timer interrupts (MTIE) enabled");
    }

    println!("✓ Integrated interrupt system test completed");
}

/// Test live timer interrupts
fn test_live_timer_interrupts() {
    println!("Setting up live timer interrupt test...");

    let test_time = CLINT_TIMER.now();
    let timer_test_target = test_time + CLINT_TIMER.ms_to_ticks(5000); // 5 seconds

    println!("Timer interrupt test setup:");
    println!("  Current time: {}", num(test_time));
    println!("  Target time: {}", num(timer_test_target));
    println!("  Interval: 5000 ms");

    // Set timer interrupt
    unsafe {
        match CLINT_TIMER.set_alarm(timer_test_target) {
            Ok(()) => println!("✓ Timer alarm set successfully"),
            Err(_) => {
                println!("✗ Failed to set timer alarm");
                return;
            }
        }
    }

    println!("Waiting for timer interrupt...");
    let mut wait_loops = 0;
    let max_wait_loops = 20;

    while wait_loops < max_wait_loops {
        let current = CLINT_TIMER.now();

        println!("Wait {}: current={}", num(wait_loops), num(current));

        if current >= timer_test_target {
            println!("✓ Timer target reached!");

            // Brief wait for interrupt processing
            for _ in 0..100000 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }

            let post_interrupt_stats = arch::current::timer::get_timer_stats();
            println!(
                "Interrupts after target: {}",
                num(post_interrupt_stats.interrupts)
            );
            break;
        }

        let remaining = timer_test_target - current;
        println!("  Remaining: {}", num(remaining));

        // Short wait
        for _ in 0..50000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }

        wait_loops += 1;
    }

    // Reset to safe state
    let safe_future = CLINT_TIMER.now() + CLINT_TIMER.ms_to_ticks(60000); // 1 minute
    unsafe {
        let _ = CLINT_TIMER.set_alarm(safe_future);
    }

    println!("✓ Live timer interrupt test completed");
}

/// Test panic system
fn test_panic_system() {
    println!("Testing panic system components...");

    // Test stack monitoring
    test_stack_monitoring();

    // Test memory checking
    test_memory_checking();

    // Test CSR state dumping
    test_csr_dumping();

    println!("✓ Panic system test completed");
    println!("Note: Actual panic test skipped to avoid system halt");
}

/// Test stack monitoring
fn test_stack_monitoring() {
    println!("Testing stack monitoring...");

    let current_sp = get_current_sp();
    println!("Current SP: {}", hex(current_sp));

    let stack_base = 0x80100000;
    let stack_used = stack_base - current_sp;
    println!("Stack used: {} bytes", num(stack_used as u64));

    if current_sp >= 0x80000000 && current_sp < 0x80100000 {
        println!("✓ Stack pointer in valid range");
    } else {
        println!("✗ Stack pointer out of range");
    }
}

/// Test memory checking
fn test_memory_checking() {
    println!("Testing memory checking...");

    let test_array = [1u64, 2u64, 3u64, 4u64];
    let ptr = test_array.as_ptr() as usize;

    println!("Test array address: {}", hex(ptr));

    let ram_start = 0x80000000;
    let ram_end = 0x88000000;

    if ptr >= ram_start && ptr < ram_end {
        println!("✓ Address in valid RAM range");
    } else {
        println!("⚠ Address outside RAM range (stack/heap)");
    }
}

/// Test CSR dumping
fn test_csr_dumping() {
    println!("Testing CSR state dumping...");

    let mstatus = arch::csr::read_mstatus();
    let mie = arch::csr::read_mie();
    let mtvec = arch::csr::read_mtvec();

    println!("Current CSR state:");
    println!("  mstatus: {}", hex(mstatus));
    println!("  mie: {}", hex(mie));
    println!("  mtvec: {}", hex(mtvec));

    let global_ie = (mstatus >> 3) & 1;
    let mtie = (mie >> 7) & 1;
    let msie = (mie >> 3) & 1;

    println!("bit field analysis:");
    println!("  global ie: {}", num(global_ie as u64));
    println!("  timer ie: {}", num(mtie as u64));
    println!("  sw ie: {}", num(msie as u64));
}

/// Main system loop
fn main_system_loop() -> ! {
    println!("Starting main system loop with integrated timers...");

    let mut integrated_counter = 0u64;
    let mut test_cycle = 0u64;
    let loop_start_time = CLINT_TIMER.now();

    loop {
        integrated_counter = integrated_counter.wrapping_add(1);

        if integrated_counter % 10000000 == 0 {
            println!("Loop count: {}", num(integrated_counter));

            test_cycle += 1;
            let current_time_ms = utils::current_time_ms();

            // Display periodic status
            match test_cycle % 6 {
                1 => {
                    // System status
                    println!("=== SYSTEM STATUS ===");
                    println!("Current time: {} ms", num(current_time_ms));
                    let uptime_seconds = current_time_ms / 1000;
                    println!("Uptime: {} seconds", num(uptime_seconds));
                }
                2 => {
                    // Timer statistics
                    println!("=== TIMER STATISTICS ===");
                    let stats = arch::current::timer::get_timer_stats();
                    println!("Timer interrupts: {}", num(stats.interrupts));
                    println!("Alarms set: {}", num(stats.alarms_set));
                    if stats.errors > 0 {
                        println!("Errors: {}", num(stats.errors));
                    }
                }
                3 => {
                    // Software interrupt test
                    if test_cycle <= 20 {
                        println!("Testing yield (SW interrupt)...");
                        match interrupt::yield_cpu_relaxed() {
                            Ok(()) => println!("✓ Yield OK"),
                            Err(e) => println!("⚠ Yield failed: {}", str(e)),
                        }
                    }
                }
                4 => {
                    // Trap test
                    if test_cycle <= 30 {
                        println!("Testing ecall...");
                        trap::test_ecall_safe();
                        println!("✓ Ecall OK");
                    }
                }
                5 => {
                    // Performance measurement
                    println!("=== PERFORMANCE TEST ===");
                    let (result, exec_time) = utils::measure_time(|| {
                        let mut sum = 0u64;
                        for i in 0..10000 {
                            sum += i;
                        }
                        sum
                    });
                    println!("Computation: {} in {} ms", num(result), num(exec_time));
                }
                0 => {
                    // Comprehensive status
                    if test_cycle % 12 == 0 {
                        println!("=== COMPREHENSIVE STATUS ===");

                        // Display system info
                        system::show_info();

                        // Display interrupt statistics
                        interrupt::display_statistics();

                        // Calculate total uptime
                        let total_elapsed = CLINT_TIMER.now() - loop_start_time;
                        let uptime_ms = CLINT_TIMER.ticks_to_ms(total_elapsed);
                        println!("Total uptime: {} ms", num(uptime_ms));
                    }
                }
                _ => {}
            }

            // Reset cycle periodically
            if test_cycle > 60 {
                test_cycle = 0;
                println!("=== CYCLE RESET - SYSTEM STABLE ===");

                // Final comprehensive report
                println!("Final system report:");
                let final_stats = arch::current::timer::get_timer_stats();
                let final_time_ms = utils::current_time_ms();

                println!("  Total runtime: {} seconds", num(final_time_ms / 1000));
                println!("  Timer interrupts: {}", num(final_stats.interrupts));
                println!("  Loop iterations: {}", num(integrated_counter));

                if final_stats.errors == 0 {
                    println!("✓ No errors detected - system running perfectly");
                } else {
                    println!("⚠ {} errors detected", num(final_stats.errors));
                }
            }
        }

        unsafe {
            core::arch::asm!("nop");
        }
    }
}

/// Helper functions
fn read_mhartid() -> u64 {
    let mut val: u64;
    unsafe {
        core::arch::asm!("csrr {}, mhartid", out(reg) val);
    }
    val
}

fn get_current_sp() -> usize {
    let mut sp: usize;
    unsafe {
        core::arch::asm!("mv {}, sp", out(reg) sp);
    }
    sp
}

/// System diagnostics function
pub fn system_diagnostics() {
    println!("=== SYSTEM DIAGNOSTICS ===");

    // Hardware information
    let mhartid = read_mhartid();
    println!("Hart ID: {}", num(mhartid));

    // Timer information
    let current_time = CLINT_TIMER.now();
    let current_ms = utils::current_time_ms();
    let stats = arch::current::timer::get_timer_stats();

    println!("Timer status:");
    println!("  Current ticks: {}", num(current_time));
    println!("  Current time: {} ms", num(current_ms));
    println!("  Timer interrupts: {}", num(stats.interrupts));

    // Memory information
    let current_sp = get_current_sp();
    let stack_used = 0x80100000 - current_sp;
    println!("Memory status:");
    println!("  Stack used: {} bytes", num(stack_used as u64));

    // Interrupt statistics
    let (sw_interrupts, yields, handlers, errors) = interrupt::get_statistics();
    println!("Interrupt status:");
    println!("  SW interrupts: {}", num(sw_interrupts));
    println!("  Yield calls: {}", num(yields));
    println!("  Handler calls: {}", num(handlers));
    println!("  Errors: {}", num(errors));

    println!("=== DIAGNOSTICS COMPLETE ===");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::enhanced_panic_handler(info)
}
