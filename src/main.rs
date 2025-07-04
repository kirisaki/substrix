#![no_std]
#![no_main]

use core::panic::PanicInfo;

// UART0 address for QEMU virt machine
const UART0: *mut u8 = 0x1000_0000 as *mut u8;

// Research findings: format_args! cannot be used in this environment
// Only macro-based implementation works reliably

macro_rules! print {
    ($s:expr) => {
        for b in $s.bytes() {
            unsafe {
                core::ptr::write_volatile(UART0, b);
            }
        }
    };
}

macro_rules! println {
    () => {
        unsafe {
            core::ptr::write_volatile(UART0, b'\n');
        }
    };
    ($s:expr) => {{
        print!($s);
        unsafe {
            core::ptr::write_volatile(UART0, b'\n');
        }
    }};
}

// Number printing (supports 0-99)
macro_rules! print_number {
    ($n:expr) => {{
        let num = $n;
        if num == 0 {
            unsafe {
                core::ptr::write_volatile(UART0, b'0');
            }
        } else if num < 10 {
            unsafe {
                core::ptr::write_volatile(UART0, (num as u8) + b'0');
            }
        } else if num < 100 {
            let tens = (num / 10) as u8 + b'0';
            let ones = (num % 10) as u8 + b'0';
            unsafe {
                core::ptr::write_volatile(UART0, tens);
                core::ptr::write_volatile(UART0, ones);
            }
        } else {
            print!("??"); // Display ?? for numbers >= 100
        }
    }};
}

// Convenience macro for printing message + number
macro_rules! println_number {
    ($msg:expr, $num:expr) => {{
        print!($msg);
        print_number!($num);
        println!();
    }};
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
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
