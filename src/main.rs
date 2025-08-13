#![no_std]
#![no_main]

#[macro_use]
mod console;

mod arch;
mod trap;

pub const UART0: *mut u8 = 0x1000_0000 as *mut u8;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("Hello, UART!");
    println!("Substrix OS booting...");

    println_number!("count: ", 12345);
    println_hex!("address: ", 0xDEADBEEF as u32);

    let my_var = 42;
    debug!(my_var);
    debug_hex!(my_var);

    println_hex!("UART base: ", 0x10000000);
    println_hex!("RAM start: ", 0x80000000 as u32);

    // Initialize trap handler
    trap::init_trap();

    // Test ecall
    trap::test_ecall();

    println!("Boot complete!");

    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC occurred!");
    if let Some(location) = info.location() {
        print!("Location: ");
        print!(location.file());
        print!(":");
        print_number!(location.line());
        println!();
    }
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
