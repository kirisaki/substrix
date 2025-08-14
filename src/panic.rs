// RISC-V Enhanced Panic Handler (Fixed Version)
// 詳細なデバッグ情報とシステム状態ダンプ機能

use crate::{arch::csr, panic_print, panic_print_hex, panic_print_number, panic_println, UART0};
use core::panic::PanicInfo;

/// パニック時のシステム状態
#[derive(Clone, Copy)]
pub struct PanicState {
    pub mstatus: usize,
    pub mcause: usize,
    pub mepc: usize,
    pub mtvec: usize,
    pub mie: usize,
    pub mip: usize,
    pub sp: usize,
    pub ra: usize,
}

/// パニック統計（静的変数として保持）
static mut PANIC_COUNT: u32 = 0;
static mut LAST_PANIC_PC: usize = 0;

/// 拡張パニックハンドラ
pub fn enhanced_panic_handler(info: &PanicInfo) -> ! {
    // 割り込みを無効化してパニック処理を安全に実行
    unsafe {
        csr::disable_global_interrupts();
    }

    // パニック統計を更新
    unsafe {
        PANIC_COUNT += 1;
    }

    // パニックヘッダーの出力
    print_panic_header();

    // パニック情報の詳細出力
    print_panic_info(info);

    // システム状態のダンプ
    let panic_state = capture_system_state();
    print_system_state(&panic_state);

    // スタック情報の出力
    print_stack_info();

    // トラップ情報の解析
    analyze_trap_cause(&panic_state);

    // パニック統計の表示
    print_panic_statistics();

    // メモリ状況の簡易確認
    print_memory_status();

    // 最終メッセージ
    print_final_message();

    // システムを安全に停止
    halt_system()
}

/// パニックヘッダーの出力
fn print_panic_header() {
    panic_println!();
    panic_println!("=====================================");
    panic_println!("        KERNEL PANIC DETECTED        ");
    panic_println!("=====================================");
    panic_println!();
}

/// パニック情報の詳細出力
fn print_panic_info(info: &PanicInfo) {
    panic_println!("=== PANIC INFORMATION ===");

    // パニックメッセージ（PanicMessage型の安全な処理）
    panic_print!("Message: ");
    let message = info.message();
    // PanicMessageを文字列として出力する安全な方法
    if let Some(s) = message.as_str() {
        panic_println!(s);
    } else {
        // フォーマット引数が含まれる場合
        panic_println!("(formatted message - cannot display safely)");
    }

    // ファイル・行番号情報
    if let Some(location) = info.location() {
        panic_print!("Location: ");
        panic_print!(location.file());
        panic_print!(":");
        panic_print_number!(location.line() as u64);
        panic_print!(":");
        panic_print_number!(location.column() as u64);
        panic_println!();

        // ファイル名のみを抽出して表示
        let file_name = location.file().split('/').last().unwrap_or("unknown");
        panic_print!("File: ");
        panic_println!(file_name);
    } else {
        panic_println!("Location: (unknown)");
    }

    panic_println!();
}

/// システム状態のキャプチャ
fn capture_system_state() -> PanicState {
    // スタックポインタとリターンアドレスの取得
    let mut sp_val: usize;
    let mut ra_val: usize;

    unsafe {
        core::arch::asm!("mv {}, sp", out(reg) sp_val);
        core::arch::asm!("mv {}, ra", out(reg) ra_val);

        // 最後のPC値を記録
        LAST_PANIC_PC = csr::read_mepc();
    }

    PanicState {
        mstatus: csr::read_mstatus(),
        mcause: csr::read_mcause(),
        mepc: csr::read_mepc(),
        mtvec: csr::read_mtvec(),
        mie: csr::read_mie(),
        mip: read_mip(),
        sp: sp_val,
        ra: ra_val,
    }
}

/// MIP (Machine Interrupt Pending) レジスタ読み取り
fn read_mip() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mip", out(reg) val);
    }
    val
}

/// システム状態の詳細出力
fn print_system_state(state: &PanicState) {
    panic_println!("=== SYSTEM STATE DUMP ===");

    // CSRレジスタ
    panic_println!("Control and Status Registers:");
    panic_print!("  mstatus: ");
    panic_print_hex!(state.mstatus);
    panic_println!();

    panic_print!("  mcause:  ");
    panic_print_hex!(state.mcause);
    panic_println!();

    panic_print!("  mepc:    ");
    panic_print_hex!(state.mepc);
    panic_println!();

    panic_print!("  mtvec:   ");
    panic_print_hex!(state.mtvec);
    panic_println!();

    panic_print!("  mie:     ");
    panic_print_hex!(state.mie);
    panic_println!();

    panic_print!("  mip:     ");
    panic_print_hex!(state.mip);
    panic_println!();

    // 基本レジスタ
    panic_println!("General Registers:");
    panic_print!("  sp:      ");
    panic_print_hex!(state.sp);
    panic_println!();

    panic_print!("  ra:      ");
    panic_print_hex!(state.ra);
    panic_println!();

    // mstatusの詳細解析
    analyze_mstatus(state.mstatus);

    panic_println!();
}

/// mstatusレジスタの詳細解析
fn analyze_mstatus(mstatus: usize) {
    panic_println!("mstatus Analysis:");

    let mie = (mstatus >> 3) & 1;
    let mpie = (mstatus >> 7) & 1;
    let mpp = (mstatus >> 11) & 3;

    panic_print!("  MIE (Global Interrupt Enable): ");
    if mie != 0 {
        panic_println!("ENABLED");
    } else {
        panic_println!("DISABLED");
    }

    panic_print!("  MPIE (Previous Interrupt Enable): ");
    if mpie != 0 {
        panic_println!("ENABLED");
    } else {
        panic_println!("DISABLED");
    }

    panic_print!("  MPP (Previous Privilege): ");
    if mpp == 0 {
        panic_println!("User");
    } else if mpp == 1 {
        panic_println!("Supervisor");
    } else if mpp == 3 {
        panic_println!("Machine");
    } else {
        panic_println!("Reserved");
    }
}

/// スタック情報の出力
fn print_stack_info() {
    panic_println!("=== STACK INFORMATION ===");

    let sp = {
        let mut val: usize;
        unsafe {
            core::arch::asm!("mv {}, sp", out(reg) val);
        }
        val
    };

    panic_print!("Current SP: ");
    panic_print_hex!(sp);
    panic_println!();

    // スタック範囲の確認
    let ram_start = 0x80000000;
    let ram_end = 0x88000000; // 128MB
    let stack_start = 0x80100000; // boot.sで設定されたスタック

    panic_print!("Stack base: ");
    panic_print_hex!(stack_start);
    panic_println!();

    panic_print!("RAM range:  ");
    panic_print_hex!(ram_start);
    panic_print!(" to ");
    panic_print_hex!(ram_end);
    panic_println!();

    // スタックの妥当性チェック
    if sp >= ram_start && sp < ram_end {
        panic_println!("Stack: ✓ Valid range");

        let stack_used = stack_start - sp;
        panic_print!("Stack used: ");
        panic_print_number!(stack_used as u64);
        panic_println!(" bytes");

        if stack_used > 0x10000 {
            // 64KB
            panic_println!("⚠ Stack usage high");
        }
    } else {
        panic_println!("Stack: ✗ CORRUPTED!");
    }

    // スタックの一部をダンプ（安全に）
    print_stack_dump(sp);

    panic_println!();
}

/// スタックダンプ（安全版）
fn print_stack_dump(sp: usize) {
    panic_println!("Stack dump (last 8 words):");

    // 8ワード（64バイト）をダンプ
    for i in 0..8 {
        let addr = sp + (i * 8);

        // アドレスの妥当性チェック
        if addr >= 0x80000000 && addr < 0x88000000 {
            let value = unsafe { core::ptr::read_volatile(addr as *const u64) };

            panic_print!("  ");
            panic_print_hex!(addr);
            panic_print!(": ");
            panic_print_hex!(value as usize);
            panic_println!();
        } else {
            panic_println!("  (stack range exceeded)");
            break;
        }
    }
}

/// トラップ原因の解析
fn analyze_trap_cause(state: &PanicState) {
    panic_println!("=== TRAP ANALYSIS ===");

    let mcause = state.mcause;
    let interrupt = (mcause >> 63) != 0;
    let exception_code = mcause & 0x7FFFFFFFFFFFFFFF;

    panic_print!("mcause: ");
    panic_print_hex!(mcause);

    if interrupt {
        panic_println!(" (INTERRUPT)");
        panic_print!("Interrupt type: ");

        if exception_code == 1 {
            panic_println!("Supervisor software interrupt");
        } else if exception_code == 3 {
            panic_println!("Machine software interrupt");
        } else if exception_code == 5 {
            panic_println!("Supervisor timer interrupt");
        } else if exception_code == 7 {
            panic_println!("Machine timer interrupt");
        } else if exception_code == 9 {
            panic_println!("Supervisor external interrupt");
        } else if exception_code == 11 {
            panic_println!("Machine external interrupt");
        } else {
            panic_print!("Unknown interrupt (");
            panic_print_number!(exception_code as u64);
            panic_println!(")");
        }
    } else {
        panic_println!(" (EXCEPTION)");
        panic_print!("Exception type: ");

        if exception_code == 0 {
            panic_println!("Instruction address misaligned");
        } else if exception_code == 1 {
            panic_println!("Instruction access fault");
        } else if exception_code == 2 {
            panic_println!("Illegal instruction");
        } else if exception_code == 3 {
            panic_println!("Breakpoint");
        } else if exception_code == 4 {
            panic_println!("Load address misaligned");
        } else if exception_code == 5 {
            panic_println!("Load access fault");
        } else if exception_code == 6 {
            panic_println!("Store/AMO address misaligned");
        } else if exception_code == 7 {
            panic_println!("Store/AMO access fault");
        } else if exception_code == 8 {
            panic_println!("Environment call from U-mode");
        } else if exception_code == 9 {
            panic_println!("Environment call from S-mode");
        } else if exception_code == 11 {
            panic_println!("Environment call from M-mode");
        } else if exception_code == 12 {
            panic_println!("Instruction page fault");
        } else if exception_code == 13 {
            panic_println!("Load page fault");
        } else if exception_code == 15 {
            panic_println!("Store/AMO page fault");
        } else {
            panic_print!("Unknown exception (");
            panic_print_number!(exception_code as u64);
            panic_println!(")");
        }
    }

    // メモリアクセス関連の例外の場合、詳細情報
    if !interrupt && (exception_code == 1 || exception_code == 5 || exception_code == 7) {
        panic_println!("Memory access fault detected!");
        panic_print!("Fault address (mepc): ");
        panic_print_hex!(state.mepc);
        panic_println!();

        // アドレスの妥当性チェック
        if state.mepc >= 0x80000000 && state.mepc < 0x88000000 {
            panic_println!("Fault address is in valid RAM range");
        } else {
            panic_println!("⚠ Fault address is OUTSIDE valid RAM range!");
        }
    }

    panic_println!();
}

/// パニック統計の表示
fn print_panic_statistics() {
    panic_println!("=== PANIC STATISTICS ===");

    unsafe {
        panic_print!("Panic count: ");
        panic_print_number!(PANIC_COUNT as u64);
        panic_println!();

        if PANIC_COUNT > 1 {
            panic_println!("⚠ Multiple panics detected!");
            panic_print!("Last panic PC: ");
            panic_print_hex!(LAST_PANIC_PC);
            panic_println!();
        }
    }

    panic_println!();
}

/// メモリ状況の簡易確認
fn print_memory_status() {
    panic_println!("=== MEMORY STATUS ===");

    // 外部リンカシンボル
    extern "C" {
        static __bss_start: u8;
        static __bss_end: u8;
        static __data_start: u8;
        static __data_end: u8;
    }

    unsafe {
        let bss_start = &__bss_start as *const u8 as usize;
        let bss_end = &__bss_end as *const u8 as usize;
        let data_start = &__data_start as *const u8 as usize;
        let data_end = &__data_end as *const u8 as usize;

        panic_println!("Memory layout:");
        panic_print!("  .data start: ");
        panic_print_hex!(data_start);
        panic_println!();

        panic_print!("  .data end:   ");
        panic_print_hex!(data_end);
        panic_println!();

        panic_print!("  .bss start:  ");
        panic_print_hex!(bss_start);
        panic_println!();

        panic_print!("  .bss end:    ");
        panic_print_hex!(bss_end);
        panic_println!();

        let data_size = data_end - data_start;
        let bss_size = bss_end - bss_start;

        panic_print!("  .data size:  ");
        panic_print_number!(data_size as u64);
        panic_println!(" bytes");

        panic_print!("  .bss size:   ");
        panic_print_number!(bss_size as u64);
        panic_println!(" bytes");
    }

    panic_println!();
}

/// 最終メッセージ
fn print_final_message() {
    panic_println!("=====================================");
    panic_println!("         SYSTEM HALTED               ");
    panic_println!("=====================================");
    panic_println!();
    panic_println!("Reason: Unrecoverable panic");
    panic_println!("Action: System stopped for safety");
    panic_println!("Note:   Restart required");
    panic_println!();
}

/// システムの安全停止
pub fn halt_system() -> ! {
    // 全ての割り込みを無効化
    unsafe {
        csr::disable_global_interrupts();
        csr::write_mie(0);
    }

    // タイマを停止（無限に先の時間に設定）
    let far_future = u64::MAX;
    unsafe {
        let mtimecmp_addr = 0x2004000 as *mut u64;
        core::ptr::write_volatile(mtimecmp_addr, far_future);
    }

    // MSIPもクリア
    unsafe {
        let msip_addr = 0x2000000 as *mut u32;
        core::ptr::write_volatile(msip_addr, 0);
    }

    // 最終的な停止ループ
    loop {
        unsafe {
            // WFI (Wait For Interrupt) - 電力節約
            core::arch::asm!("wfi");
        }
    }
}

/// デバッグ用パニック（手動トリガー）
pub fn debug_panic(message: &str) -> ! {
    panic!("Debug panic: {}", message);
}

/// アサーション失敗時のパニック
#[track_caller]
pub fn assertion_failed(condition: &str, file: &str, line: u32) -> ! {
    panic!("Assertion failed: {} at {}:{}", condition, file, line);
}

/// メモリ破損検出時のパニック
pub fn memory_corruption_panic(address: usize, expected: u64, actual: u64) -> ! {
    panic!(
        "Memory corruption at {:#x}: expected {:#x}, found {:#x}",
        address, expected, actual
    );
}

/// スタックオーバーフロー検出時のパニック
pub fn stack_overflow_panic(sp: usize, limit: usize) -> ! {
    panic!("Stack overflow: SP={:#x}, limit={:#x}", sp, limit);
}

/// より詳細なアサーションマクロ
#[macro_export]
macro_rules! kassert {
    ($cond:expr) => {
        if !($cond) {
            $crate::panic::assertion_failed(stringify!($cond), file!(), line!());
        }
    };
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            panic!("Assertion failed: {} - {}", stringify!($cond), $msg);
        }
    };
}

/// デバッグ専用のソフトパニック（開発時用）
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug_panic {
    ($msg:expr) => {
        $crate::panic::debug_panic($msg)
    };
    ($fmt:expr, $($args:tt)*) => {
        $crate::panic::debug_panic(&format!($fmt, $($args)*))
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug_panic {
    ($msg:expr) => {};
    ($fmt:expr, $($args:tt)*) => {};
}
