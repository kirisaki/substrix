// Advanced Debug & Recovery System
// スタックトレース、メモリプロテクション、ソフトリセット

use crate::{arch::csr, print, println, println_hex, println_number, UART0};

/// デバッグ情報の詳細レベル
#[derive(Clone, Copy, PartialEq)]
pub enum DebugLevel {
    Minimal,  // 最小限の情報
    Standard, // 標準的な情報
    Verbose,  // 詳細な情報
    Full,     // 全ての情報
}

/// システム復旧オプション
#[derive(Clone, Copy)]
pub enum RecoveryOption {
    Halt,           // システム停止
    SoftReset,      // ソフトリセット
    SafeMode,       // セーフモード
    ContinueUnsafe, // 危険だが継続
}

/// 簡易スタックトレース（リターンアドレスを辿る）
pub fn print_stack_trace(max_depth: usize) {
    println!("=== STACK TRACE ===");

    let mut ra = get_return_address();
    let mut fp = get_frame_pointer();
    let mut depth = 0;

    println!("Call stack (approximate):");

    while depth < max_depth && is_valid_address(ra) && is_valid_address(fp) {
        print!("  #");
        print_number!(depth as u64);
        print!(": ");
        print_hex!(ra);

        // 関数名の推定（簡易版）
        if let Some(name) = guess_function_name(ra) {
            print!(" <");
            print!(name);
            print!(">");
        }
        println!();

        // 次のフレームに移動（簡易版）
        if fp >= 0x80000000 && fp < 0x80100000 {
            // フレームポインタからリターンアドレスを取得
            let next_ra = unsafe {
                if fp + 8 < 0x80100000 {
                    core::ptr::read_volatile((fp + 8) as *const usize)
                } else {
                    0
                }
            };

            let next_fp = unsafe {
                if fp < 0x80100000 {
                    core::ptr::read_volatile(fp as *const usize)
                } else {
                    0
                }
            };

            ra = next_ra;
            fp = next_fp;
        } else {
            break;
        }

        depth += 1;
    }

    if depth == 0 {
        println!("  (unable to trace stack)");
    }

    println!();
}

/// リターンアドレスの取得
fn get_return_address() -> usize {
    let mut ra: usize;
    unsafe {
        core::arch::asm!("mv {}, ra", out(reg) ra);
    }
    ra
}

/// フレームポインタの取得
fn get_frame_pointer() -> usize {
    let mut fp: usize;
    unsafe {
        core::arch::asm!("mv {}, fp", out(reg) fp);
    }
    fp
}

/// アドレスの妥当性チェック
fn is_valid_address(addr: usize) -> bool {
    // RAM範囲内かチェック
    addr >= 0x80000000 && addr < 0x88000000 && addr % 4 == 0
}

/// 関数名の推定（既知のアドレス範囲から）
fn guess_function_name(addr: usize) -> Option<&'static str> {
    // Rustのmain関数やブート関連のおおよその範囲
    // 実際の実装では、.mapファイルやデバッグ情報を使用

    extern "C" {
        fn _start();
        fn rust_main();
    }

    let start_addr = _start as usize;
    let main_addr = rust_main as usize;

    // 簡易的な範囲推定
    if addr >= start_addr && addr < start_addr + 0x100 {
        Some("_start")
    } else if addr >= main_addr && addr < main_addr + 0x1000 {
        Some("rust_main")
    } else if addr >= 0x80000000 && addr < 0x80001000 {
        Some("boot_section")
    } else if addr >= 0x80001000 && addr < 0x80010000 {
        Some("kernel_code")
    } else {
        None
    }
}

/// メモリプロテクション - 特定範囲の監視
pub struct MemoryGuard {
    start: usize,
    end: usize,
    checksum: u32,
    name: &'static str,
}

impl MemoryGuard {
    /// 新しいメモリガードを作成
    pub fn new(start: usize, end: usize, name: &'static str) -> Self {
        let checksum = calculate_checksum(start, end);
        Self {
            start,
            end,
            checksum,
            name,
        }
    }

    /// メモリの整合性をチェック
    pub fn check(&self) -> bool {
        let current_checksum = calculate_checksum(self.start, self.end);
        current_checksum == self.checksum
    }

    /// チェックサムを更新（意図的な変更後）
    pub fn update_checksum(&mut self) {
        self.checksum = calculate_checksum(self.start, self.end);
    }

    /// 破損を検出した場合のアクション
    pub fn handle_corruption(&self) {
        println!("Memory corruption detected!");
        print!("Protected region: ");
        println!(self.name);
        println_hex!("Start: ", self.start);
        println_hex!("End: ", self.end);

        // 詳細な分析
        analyze_memory_corruption(self.start, self.end);

        panic!("Memory corruption in protected region: {}", self.name);
    }
}

/// 簡易チェックサム計算
fn calculate_checksum(start: usize, end: usize) -> u32 {
    let mut sum = 0u32;
    let mut addr = start;

    while addr < end && addr + 4 <= end {
        if is_valid_address(addr) {
            let value = unsafe { core::ptr::read_volatile(addr as *const u32) };
            sum = sum.wrapping_add(value);
        }
        addr += 4;
    }

    sum
}

/// メモリ破損の詳細分析
fn analyze_memory_corruption(start: usize, end: usize) {
    println!("Analyzing memory corruption...");

    let mut corrupted_bytes = 0;
    let mut total_bytes = 0;

    for addr in (start..end).step_by(4) {
        if addr + 4 <= end && is_valid_address(addr) {
            total_bytes += 4;

            // 簡易的な破損検出（ゼロでない予期しない値）
            let value = unsafe { core::ptr::read_volatile(addr as *const u32) };

            // 特定のパターンを破損として検出
            if value == 0xDEADBEEF || value == 0xBADCAFE0 {
                corrupted_bytes += 4;
                print!("Corruption at ");
                print_hex!(addr);
                print!(": ");
                print_hex!(value as usize);
                println!();
            }
        }
    }

    println_number!("Total bytes checked: ", total_bytes as u64);
    println_number!("Corrupted bytes: ", corrupted_bytes as u64);

    if corrupted_bytes > 0 {
        let corruption_rate = (corrupted_bytes * 100) / total_bytes;
        println_number!("Corruption rate: ", corruption_rate as u64);
        println!("%");
    }
}

/// ソフトリセット機能
pub fn soft_reset() -> ! {
    println!("=== PERFORMING SOFT RESET ===");

    // Step 1: 全ての割り込みを無効化
    println!("Disabling interrupts...");
    unsafe {
        csr::disable_global_interrupts();
        csr::write_mie(0);
    }

    // Step 2: ハードウェアの安全な停止
    println!("Stopping hardware...");
    stop_all_hardware();

    // Step 3: メモリの部分的なクリア（重要でない部分のみ）
    println!("Clearing non-critical memory...");
    clear_temporary_memory();

    // Step 4: CSRの再初期化
    println!("Reinitializing CSRs...");
    reinitialize_csrs();

    // Step 5: ジャンプして再開
    println!("Jumping to reset vector...");

    extern "C" {
        fn _start();
    }

    let reset_addr = _start as usize;
    println_hex!("Reset address: ", reset_addr);

    unsafe {
        // mepcに_startアドレスを設定
        csr::write_mepc(reset_addr);

        // mretで_startにジャンプ
        core::arch::asm!("mret");
    }

    // ここには到達しないはず
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// 全ハードウェアの安全停止
fn stop_all_hardware() {
    // タイマの停止
    unsafe {
        let mtimecmp_addr = 0x2004000 as *mut u64;
        core::ptr::write_volatile(mtimecmp_addr, u64::MAX);
    }

    // MSIPのクリア
    unsafe {
        let msip_addr = 0x2000000 as *mut u32;
        core::ptr::write_volatile(msip_addr, 0);
    }

    // その他のペリフェラル（必要に応じて追加）
}

/// 一時的なメモリのクリア
fn clear_temporary_memory() {
    // スタックの上部をクリア（現在のSPより上の部分）
    let current_sp = {
        let mut sp: usize;
        unsafe {
            core::arch::asm!("mv {}, sp", out(reg) sp);
        }
        sp
    };

    let stack_top = 0x80100000;

    if current_sp < stack_top {
        let clear_size = stack_top - current_sp;
        println_number!("Clearing stack: ", clear_size as u64);
        println!(" bytes");

        unsafe {
            core::ptr::write_bytes(current_sp as *mut u8, 0, clear_size);
        }
    }
}

/// CSRの再初期化
fn reinitialize_csrs() {
    unsafe {
        // 基本的なCSRをクリア
        csr::write_mepc(0);

        // mstatusを初期状態に
        let initial_mstatus = (3 << 11); // MPP = Machine mode
        csr::write_mstatus(initial_mstatus);

        // mieをクリア
        csr::write_mie(0);
    }
}

/// セーフモード（制限機能での継続）
pub fn enter_safe_mode() {
    println!("=== ENTERING SAFE MODE ===");

    // 割り込みを無効化
    unsafe {
        csr::disable_global_interrupts();
    }

    // タイマを停止
    stop_all_hardware();

    println!("Safe mode activated:");
    println!("- All interrupts disabled");
    println!("- Hardware stopped");
    println!("- Limited functionality available");
    println!();

    // セーフモードでの基本機能
    safe_mode_shell();
}

/// セーフモードの簡易シェル
fn safe_mode_shell() {
    println!("Safe Mode Shell - Limited Commands:");
    println!("Commands: status, memory, reset, halt");
    println!("Type 'help' for more information");
    println!();

    let mut command_count = 0;
    let max_commands = 10; // 無限ループ防止

    while command_count < max_commands {
        print!("safe> ");

        // 実際のキーボード入力はないので、デモ用のコマンドを実行
        let demo_commands = ["status", "memory", "reset"];
        let cmd = demo_commands[command_count % 3];

        println!(cmd);

        match cmd {
            "status" => {
                show_safe_mode_status();
            }
            "memory" => {
                show_memory_info();
            }
            "reset" => {
                println!("Initiating soft reset...");
                soft_reset();
            }
            "halt" => {
                println!("Halting system...");
                break;
            }
            _ => {
                println!("Unknown command: {}", cmd);
            }
        }

        command_count += 1;

        // デモ用の遅延
        for _ in 0..10000000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    println!("Safe mode shell exiting...");
}

/// セーフモード状態の表示
fn show_safe_mode_status() {
    println!("=== SAFE MODE STATUS ===");

    let mstatus = csr::read_mstatus();
    let mie = csr::read_mie();
    let mtvec = csr::read_mtvec();

    println_hex!("mstatus: ", mstatus);
    println_hex!("mie:     ", mie);
    println_hex!("mtvec:   ", mtvec);

    let global_ie = (mstatus >> 3) & 1;
    if global_ie == 0 {
        println!("✓ Interrupts safely disabled");
    } else {
        println!("⚠ Interrupts still enabled");
    }

    let current_sp = {
        let mut sp: usize;
        unsafe {
            core::arch::asm!("mv {}, sp", out(reg) sp);
        }
        sp
    };

    println_hex!("Current SP: ", current_sp);

    if current_sp >= 0x80000000 && current_sp < 0x80100000 {
        println!("✓ Stack in valid range");
    } else {
        println!("⚠ Stack may be corrupted");
    }
}

/// メモリ情報の表示（セーフモード用）
fn show_memory_info() {
    println!("=== MEMORY INFORMATION ===");

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

        println_hex!(".data start: ", data_start);
        println_hex!(".data end:   ", data_end);
        println_hex!(".bss start:  ", bss_start);
        println_hex!(".bss end:    ", bss_end);

        let total_used = (data_end - data_start) + (bss_end - bss_start);
        println_number!("Total used:  ", total_used as u64);
        println!(" bytes");
    }

    let current_sp = {
        let mut sp: usize;
        unsafe {
            core::arch::asm!("mv {}, sp", out(reg) sp);
        }
        sp
    };

    let stack_used = 0x80100000 - current_sp;
    println_number!("Stack used:  ", stack_used as u64);
    println!(" bytes");
}

/// 復旧オプションの決定
pub fn decide_recovery_action(error_type: &str, severity: u8) -> RecoveryOption {
    println!("Deciding recovery action...");
    print!("Error type: ");
    println!(error_type);
    println_number!("Severity: ", severity as u64);

    match error_type {
        "memory_corruption" => {
            if severity >= 8 {
                RecoveryOption::SoftReset
            } else if severity >= 5 {
                RecoveryOption::SafeMode
            } else {
                RecoveryOption::ContinueUnsafe
            }
        }
        "stack_overflow" => RecoveryOption::SoftReset,
        "trap_error" => RecoveryOption::SafeMode,
        "assertion_failure" => {
            if severity >= 7 {
                RecoveryOption::Halt
            } else {
                RecoveryOption::SafeMode
            }
        }
        _ => RecoveryOption::Halt,
    }
}

/// 復旧アクションの実行
pub fn execute_recovery_action(action: RecoveryOption, message: &str) -> ! {
    println!("Executing recovery action...");
    print!("Action: ");

    match action {
        RecoveryOption::Halt => {
            println!("HALT");
            println!(message);
            crate::panic::halt_system();
        }
        RecoveryOption::SoftReset => {
            println!("SOFT RESET");
            println!(message);
            soft_reset();
        }
        RecoveryOption::SafeMode => {
            println!("SAFE MODE");
            println!(message);
            enter_safe_mode();
            loop {
                unsafe {
                    core::arch::asm!("wfi");
                }
            }
        }
        RecoveryOption::ContinueUnsafe => {
            println!("CONTINUE (UNSAFE)");
            println!("⚠ WARNING: Continuing with potential system instability");
            println!(message);

            // この場合は実際には継続できないため、セーフモードに入る
            enter_safe_mode();
            loop {
                unsafe {
                    core::arch::asm!("wfi");
                }
            }
        }
    }
}

/// デバッグレベル別の情報出力
pub fn print_debug_info(level: DebugLevel, context: &str) {
    match level {
        DebugLevel::Minimal => {
            println!("DEBUG: {}", context);
        }
        DebugLevel::Standard => {
            println!("DEBUG: {}", context);
            let mstatus = csr::read_mstatus();
            println_hex!("mstatus: ", mstatus);
        }
        DebugLevel::Verbose => {
            println!("DEBUG: {}", context);
            let mstatus = csr::read_mstatus();
            let mepc = csr::read_mepc();
            let mcause = csr::read_mcause();
            println_hex!("mstatus: ", mstatus);
            println_hex!("mepc:    ", mepc);
            println_hex!("mcause:  ", mcause);
        }
        DebugLevel::Full => {
            println!("DEBUG: {}", context);
            crate::system_diagnostics();
            print_stack_trace(5);
        }
    }
}
