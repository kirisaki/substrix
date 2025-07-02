#![no_std]
#![no_main]

use core::panic::PanicInfo;

// UART0アドレス（QEMU virt machine 用）
const UART0: *mut u8 = 0x1000_0000 as *mut u8;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let msg = b"Hello, Substrix!\n";
    for &b in msg {
        unsafe {
            core::ptr::write_volatile(UART0, b);
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
