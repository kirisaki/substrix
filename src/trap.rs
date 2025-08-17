// trap.rs - Timer Integration Update
// Update the timer interrupt handling in trap.rs to use unified HAL timer

// In the existing trap.rs file, update the timer interrupt case:

// OLD CODE (in trap.rs):
// TrapCause::TimerInterrupt => {
//     // Timer interrupt processing
//     unsafe {
//         // Timer interrupt marker (debug use)
//         core::ptr::write_volatile(UART0, b'[');
//         core::ptr::write_volatile(UART0, b'T');
//         core::ptr::write_volatile(UART0, b'I');
//         core::ptr::write_volatile(UART0, b'M');
//         core::ptr::write_volatile(UART0, b']');
//     }
//
//     // Call timer handler
//     crate::timer::handle_timer_interrupt();
//
//     unsafe {
//         core::ptr::write_volatile(UART0, b'T');
//         core::ptr::write_volatile(UART0, b'\n');
//     }
// }

// NEW CODE (replace the above with this):

use crate::arch::current::timer;
use crate::{arch, println, println_hex, UART0};

// Define traps
#[derive(Debug)]
pub enum TrapCause {
    SoftwareInterrupt, // Software interrupt
    TimerInterrupt,    // Timer interrupt
    Ecall,
    Other(usize),
}

impl TrapCause {
    pub fn from_mcause(mcause: usize) -> Self {
        let interrupt = (mcause >> 63) != 0;
        let exception_code = mcause & 0x7FFFFFFFFFFFFFFF;

        if interrupt {
            match exception_code {
                3 => TrapCause::SoftwareInterrupt, // Machine software interrupt
                7 => TrapCause::TimerInterrupt,    // Machine timer interrupt
                _ => TrapCause::Other(mcause),
            }
        } else {
            match exception_code {
                11 => TrapCause::Ecall, // Environment call from M-mode
                _ => TrapCause::Other(mcause),
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_trap_handler() {
    let mcause = arch::csr::read_mcause();
    let mepc = arch::csr::read_mepc();

    let trap_cause = TrapCause::from_mcause(mcause);

    match trap_cause {
        TrapCause::SoftwareInterrupt => {
            // Software interrupt processing (existing code unchanged)
            let msip_addr = 0x2000000 as *mut u32;
            unsafe {
                // Success marker (debug use)
                core::ptr::write_volatile(UART0, b'[');
                core::ptr::write_volatile(UART0, b'S');
                core::ptr::write_volatile(UART0, b'W');
                core::ptr::write_volatile(UART0, b']');

                // Clear MSIP directly (important: prevents infinite loop)
                core::ptr::write_volatile(msip_addr, 0);

                // Completion marker
                core::ptr::write_volatile(UART0, b'S');
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::TimerInterrupt => {
            // Timer interrupt processing (UPDATED for HAL)
            unsafe {
                // Timer interrupt marker (debug use)
                core::ptr::write_volatile(UART0, b'[');
                core::ptr::write_volatile(UART0, b'T');
                core::ptr::write_volatile(UART0, b'I');
                core::ptr::write_volatile(UART0, b'M');
                core::ptr::write_volatile(UART0, b']');
            }

            // Call unified HAL timer handler
            timer::handle_timer_interrupt();

            unsafe {
                core::ptr::write_volatile(UART0, b'T');
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::Ecall => {
            // ecall processing - advance mepc to next instruction
            unsafe {
                arch::csr::write_mepc(mepc + 4);
                core::ptr::write_volatile(UART0, b'E');
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::Other(_cause) => {
            // Debug information output (existing code unchanged)
            let interrupt = (mcause >> 63) != 0;
            let exception_code = mcause & 0x7FFFFFFFFFFFFFFF;

            unsafe {
                core::ptr::write_volatile(UART0, b'?');

                // More detailed debug information
                if interrupt {
                    core::ptr::write_volatile(UART0, b'I'); // Interrupt

                    // Output exception code (hex)
                    let code_high = (exception_code >> 4) & 0xF;
                    let code_low = exception_code & 0xF;

                    let hex_high = if code_high < 10 {
                        b'0' + code_high as u8
                    } else {
                        b'a' + (code_high - 10) as u8
                    };
                    let hex_low = if code_low < 10 {
                        b'0' + code_low as u8
                    } else {
                        b'a' + (code_low - 10) as u8
                    };

                    core::ptr::write_volatile(UART0, hex_high);
                    core::ptr::write_volatile(UART0, hex_low);
                } else {
                    core::ptr::write_volatile(UART0, b'E'); // Exception

                    let code = exception_code & 0xF;
                    let hex_char = if code < 10 {
                        b'0' + code as u8
                    } else {
                        b'a' + (code - 10) as u8
                    };
                    core::ptr::write_volatile(UART0, hex_char);
                }

                // mepc details
                core::ptr::write_volatile(UART0, b'@');
                let mepc_low = (mepc >> 4) & 0xF;
                let mepc_hex = if mepc_low < 10 {
                    b'0' + mepc_low as u8
                } else {
                    b'a' + (mepc_low - 10) as u8
                };
                core::ptr::write_volatile(UART0, mepc_hex);

                core::ptr::write_volatile(UART0, b'\n');
            }

            // Emergency handling for software interrupts that come to Other case
            if interrupt && exception_code == 3 {
                let msip_addr = 0x2000000 as *mut u32;
                unsafe {
                    // Emergency processing marker
                    core::ptr::write_volatile(UART0, b'[');
                    core::ptr::write_volatile(UART0, b'E');
                    core::ptr::write_volatile(UART0, b'M');
                    core::ptr::write_volatile(UART0, b'E');
                    core::ptr::write_volatile(UART0, b'R');
                    core::ptr::write_volatile(UART0, b'G');
                    core::ptr::write_volatile(UART0, b']');

                    core::ptr::write_volatile(msip_addr, 0); // Emergency MSIP clear

                    core::ptr::write_volatile(UART0, b'S');
                    core::ptr::write_volatile(UART0, b'\n');
                }
            }
        }
    }
}

pub fn init_trap() {
    extern "C" {
        fn trap_handler();
    }

    let handler_addr = trap_handler as usize;
    unsafe {
        arch::csr::write_mtvec(handler_addr);
    }

    println!("Safe trap handler initialized (HAL timer integrated)");
    println_hex!("mtvec: ", handler_addr);
}

pub fn test_ecall_safe() {
    println!("Testing safe ecall...");
    unsafe {
        core::arch::asm!("ecall");
    }
    println!("Safe ecall returned!");
}
