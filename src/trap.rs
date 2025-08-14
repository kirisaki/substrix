use crate::println;
use crate::println_hex;
use crate::{arch, timer, UART0};

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
    let mcause = arch::csr::read_mcause();
    let mepc = arch::csr::read_mepc();

    let trap_cause = TrapCause::from_mcause(mcause);

    match trap_cause {
        TrapCause::TimerInterrupt => {
            // タイマ割り込み処理
            timer::handle_timer_interrupt();
        }
        TrapCause::Ecall => {
            // ecall処理 - mepcを次の命令に進める
            unsafe {
                arch::csr::write_mepc(mepc + 4);
            }
            unsafe {
                core::ptr::write_volatile(UART0, b'E'); // Ecall processed
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::SoftwareInterrupt => {
            unsafe {
                core::ptr::write_volatile(UART0, b'S'); // Software interrupt
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::ExternalInterrupt => {
            unsafe {
                core::ptr::write_volatile(UART0, b'X'); // External interrupt
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::Other(cause) => {
            // 未知の割り込み/例外 - 詳細情報を出力
            unsafe {
                core::ptr::write_volatile(UART0, b'?'); // Unknown
                core::ptr::write_volatile(UART0, b':');
                // causeの下位4ビットを16進数で出力
                let digit = (cause & 0xF) as u8;
                let hex_char = if digit < 10 {
                    b'0' + digit
                } else {
                    b'a' + digit - 10
                };
                core::ptr::write_volatile(UART0, hex_char);

                // メモリアクセス例外（cause = 5または7）の場合は詳細出力
                if (cause & 0x7FFFFFFFFFFFFFFF) == 5 || (cause & 0x7FFFFFFFFFFFFFFF) == 7 {
                    core::ptr::write_volatile(UART0, b'M'); // Memory fault
                                                            // mepcも出力
                    core::ptr::write_volatile(UART0, b'@');
                    let mepc_digit = ((mepc >> 12) & 0xF) as u8;
                    let mepc_hex = if mepc_digit < 10 {
                        b'0' + mepc_digit
                    } else {
                        b'a' + mepc_digit - 10
                    };
                    core::ptr::write_volatile(UART0, mepc_hex);
                }

                core::ptr::write_volatile(UART0, b'\n');
            }
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
