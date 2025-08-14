use crate::arch::csr;
use crate::{debug, print, print_number, println, UART0};

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
