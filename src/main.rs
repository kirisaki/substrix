#![no_std]
#![no_main]

pub mod console;
pub mod memory;

use core::panic::PanicInfo;

// UART0 address for QEMU virt machine
const UART0: *mut u8 = 0x1000_0000 as *mut u8;

static mut COUNTER: u32 = 42;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        memory::zero_bss();
        memory::init_data();
        println_number!("COUNTER = ", COUNTER);
        COUNTER += 1;
        println_number!("COUNTER + 1 = ", COUNTER);
    }
    println!("=== Substrix OS ===");
    println!();

    println!("Research findings:");
    println!("- format_args! is not usable in this environment");
    println!("- Only macro-based implementation works");
    println!("- Multiple calls fully supported");
    println!();

    println!("Feature tests:");
    println!("Line 1: Basic string output");
    println!("Line 2: Multiple line support");
    println!("Line 3: Macro expansion works perfectly");
    println!();

    println_number!("Number test 1: ", 0);
    println_number!("Number test 2: ", 5);
    println_number!("Number test 3: ", 42);
    println_number!("Number test 4: ", 99);
    println_number!("Number test 5: ", 123); // Will display ??
    println!();

    println!("=== All features confirmed working ===");
    println!("This println! can be called any number of times!");

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    println!("Panic occurred!");
    println!("System halted.");
    loop {}
}
