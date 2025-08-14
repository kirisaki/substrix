#![no_std]
#![no_main]

#[macro_use]
mod console;

mod arch;
mod trap;

mod test;

pub const UART0: *mut u8 = 0x1000_0000 as *mut u8;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("Substrix OS booting...");

    // Tests without trap handler
    test::run_all_tests();

    // Initialize trap handler
    trap::init_trap();

    // Test with trap
    test::run_detailed_tests();

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
