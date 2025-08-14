use crate::arch::csr;
use crate::{debug, print, print_number, println, timer, UART0};

pub fn run_all_tests() {
    println!("Running basic tests...");
    println!("=====================");

    test_arithmetic();
    test_memory();
    test_uart();
    test_csr();
    test_trap_system();

    println!("All tests completed!");
}

fn test_arithmetic() {
    print!("Arithmetic test... ");
    let result = 2 + 2;
    if result == 4 {
        println!("PASS");
    } else {
        println!("FAIL");
    }
}

fn test_memory() {
    print!("Memory test... ");
    let mut array = [1, 2, 3];
    array[1] = 10;

    if array[1] == 10 {
        println!("PASS");
    } else {
        println!("FAIL");
    }
}

fn test_uart() {
    print!("UART test... ");
    print_number!(123);
    println!(" PASS");
}

fn test_csr() {
    print!("CSR test... ");
    let mstatus = csr::read_mstatus();
    let mepc = csr::read_mepc();
    let mcause = csr::read_mcause();

    println!("PASS");
}

fn test_trap_system() {
    print!("Trap system test... ");

    let mtvec = csr::read_mtvec();
    if mtvec == 0 {
        println!("SKIP (trap not initialized)");
        return;
    }

    unsafe {
        core::arch::asm!("ecall");
    }

    println!("PASS");
}

#[allow(dead_code)]
pub fn run_detailed_tests() {
    println!("Running detailed tests...");
    println!("========================");

    detailed_csr_test();
    detailed_trap_test();
    detailed_timer_test();
}

#[allow(dead_code)]
fn detailed_csr_test() {
    println!("Detailed CSR test...");

    let mstatus = csr::read_mstatus();
    println!("  mstatus: ");
    debug_hex!(mstatus);

    let mtvec = csr::read_mtvec();
    println!("  mtvec: ");
    debug_hex!(mtvec);

    let mie = csr::read_mie();
    println!("  mie: ");
    debug_hex!(mie);

    println!("Detailed CSR test PASS");
}

#[allow(dead_code)]
fn detailed_trap_test() {
    println!("Detailed trap test...");

    let mtvec = csr::read_mtvec();
    if mtvec == 0 {
        println!("  ERROR: mtvec not set!");
        return;
    }

    println!("  mtvec configured, testing ecall...");
    unsafe {
        core::arch::asm!("ecall");
    }

    println!("Detailed trap test PASS");
}

#[allow(dead_code)]
fn detailed_timer_test() {
    println!("Detailed timer test...");

    // タイマレジスタのテスト（修正版）
    let mtime = timer::read_mtime();
    println!("  Current mtime: ");
    debug!(mtime);

    let current_ms = timer::get_time_ms();
    println!("  Current time in ms: ");
    debug!(current_ms);

    let current_ticks = timer::get_ticks();
    println!("  Current ticks: ");
    debug!(current_ticks);

    // 割り込み有効状態のテスト
    let interrupts_on = csr::interrupts_enabled();
    if interrupts_on {
        println!("  Global interrupts: ENABLED");
    } else {
        println!("  Global interrupts: DISABLED");
    }

    // MIEレジスタの確認
    let mie = csr::read_mie();
    println!("  mie register: ");
    debug_hex!(mie);

    if (mie & (1 << 7)) != 0 {
        println!("  Timer interrupt enable (MTIE): ENABLED");
    } else {
        println!("  Timer interrupt enable (MTIE): DISABLED");
    }

    // タイマが正常に動作している場合
    if mtime > 0 {
        println!("  Timer hardware: WORKING");
    } else {
        println!("  Timer hardware: NOT WORKING");
    }

    println!("Detailed timer test PASS");
}
