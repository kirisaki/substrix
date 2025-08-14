use crate::{arch, println, println_hex, UART0};

// Define traps
#[derive(Debug)]
pub enum TrapCause {
    SoftwareInterrupt, // ソフトウェア割り込みを追加
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

    // デバッグ: mcauseの詳細解析
    let interrupt = (mcause >> 63) != 0;
    let exception_code = mcause & 0x7FFFFFFFFFFFFFFF;

    let trap_cause = TrapCause::from_mcause(mcause);

    match trap_cause {
        TrapCause::SoftwareInterrupt => {
            // ソフトウェア割り込み処理（直接MSIPクリア）
            let msip_addr = 0x2000000 as *mut u32;
            unsafe {
                // 成功マーカー（デバッグ用）
                core::ptr::write_volatile(UART0, b'[');
                core::ptr::write_volatile(UART0, b'S');
                core::ptr::write_volatile(UART0, b'W');
                core::ptr::write_volatile(UART0, b']');

                // MSIP を直接クリア（重要：無限ループ防止）
                core::ptr::write_volatile(msip_addr, 0);

                // 完了マーカー
                core::ptr::write_volatile(UART0, b'S');
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::Ecall => {
            // ecall処理 - mepcを次の命令に進める
            unsafe {
                arch::csr::write_mepc(mepc + 4);
                core::ptr::write_volatile(UART0, b'E');
                core::ptr::write_volatile(UART0, b'\n');
            }
        }
        TrapCause::Other(cause) => {
            // デバッグ情報の詳細出力
            unsafe {
                core::ptr::write_volatile(UART0, b'?');

                // より詳細なデバッグ情報
                if interrupt {
                    core::ptr::write_volatile(UART0, b'I'); // Interrupt

                    // 例外コードの出力（16進）
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

                // mepcの詳細
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

            // ソフトウェア割り込みが Other ケースに来た場合の緊急処理
            if interrupt && exception_code == 3 {
                let msip_addr = 0x2000000 as *mut u32;
                unsafe {
                    // 緊急処理マーカー
                    core::ptr::write_volatile(UART0, b'[');
                    core::ptr::write_volatile(UART0, b'E');
                    core::ptr::write_volatile(UART0, b'M');
                    core::ptr::write_volatile(UART0, b'E');
                    core::ptr::write_volatile(UART0, b'R');
                    core::ptr::write_volatile(UART0, b'G');
                    core::ptr::write_volatile(UART0, b']');

                    core::ptr::write_volatile(msip_addr, 0); // 緊急MSIPクリア

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

    println!("Safe trap handler initialized (debug version)");
    println_hex!("mtvec: ", handler_addr);
}

pub fn test_ecall_safe() {
    println!("Testing safe ecall...");
    unsafe {
        core::arch::asm!("ecall");
    }
    println!("Safe ecall returned!");
}
