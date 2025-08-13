use crate::println;
use crate::println_hex;
use crate::{arch, UART0};

// Define traps
#[derive(Debug)]
pub enum TrapCause {
    SoftwareInterrupt,
    TimerInterrupt,
    ExternalInterrupt,
    Ecall,
    Other(usize),
}

impl TrapCause {
    pub fn from_mcause(mcause: usize) -> Self {
        let interrupt = (mcause >> 63) != 0;
        let exception_code = mcause & 0x7FFFFFFFFFFFFFFF;

        if interrupt {
            match exception_code {
                3 => TrapCause::SoftwareInterrupt,
                7 => TrapCause::TimerInterrupt,
                11 => TrapCause::ExternalInterrupt,
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
    // デバッグ: トラップハンドラに入ったことを示す
    unsafe {
        core::ptr::write_volatile(crate::UART0, b'R');
        core::ptr::write_volatile(crate::UART0, b'\n');
    }

    let mcause = arch::csr::read_mcause();
    let mepc = arch::csr::read_mepc();

    // シンプルな出力
    unsafe {
        core::ptr::write_volatile(crate::UART0, b'C'); // Cause
        core::ptr::write_volatile(crate::UART0, b':');
        // mcauseの下位4ビットを16進数で出力
        let digit = (mcause & 0xF) as u8;
        let hex_char = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + digit - 10
        };
        core::ptr::write_volatile(crate::UART0, hex_char);
        core::ptr::write_volatile(crate::UART0, b'\n');
    }

    // ecallの場合、mepcを進める
    if mcause == 11 {
        unsafe {
            arch::csr::write_mepc(mepc + 4);
            core::ptr::write_volatile(crate::UART0, b'O'); // OK
            core::ptr::write_volatile(crate::UART0, b'K');
            core::ptr::write_volatile(crate::UART0, b'\n');
        }
    }
}

pub fn init_trap() {
    extern "C" {
        fn trap_handler();
    }

    // mtvecにトラップハンドラのアドレスを設定
    // 下位2ビットは00（Direct mode）
    let handler_addr = trap_handler as usize;
    unsafe {
        arch::csr::write_mtvec(handler_addr);
    }

    println!("Trap handler initialized");
    println_hex!("mtvec: ", handler_addr);
}

// テスト用のecall関数
pub fn test_ecall() {
    println!("Testing ecall...");
    unsafe {
        core::arch::asm!("ecall");
    }
    println!("ecall returned successfully!");
}
