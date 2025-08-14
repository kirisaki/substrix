// RISC-V タイマ割り込み統合実装
// ソフトウェア割り込みと協調動作する安全なタイマシステム

use crate::{arch::csr, println, println_hex, println_number, UART0};

// 検証済みCLINTアドレス（MSIPテストで確認済み）
const MTIME_ADDR: *mut u64 = 0x200BFF8 as *mut u64; // 現在時刻
const MTIMECMP_ADDR: *mut u64 = 0x2004000 as *mut u64; // 比較値

// タイマ頻度（QEMU virtでは10MHz）
const TIMER_FREQ: u64 = 10_000_000;

// グローバルなtickカウンタと統計
static mut TICKS: u64 = 0;
static mut TIMER_INTERRUPT_COUNT: u64 = 0;
static mut LAST_MTIMECMP_VALUE: u64 = 0;

// エラー統計
static mut TIMER_ERRORS: u64 = 0;

/// タイマシステムの安全な初期化
pub fn init_timer() {
    println!("=== TIMER SYSTEM INITIALIZATION ===");

    // Step 1: 基本的なMTIME読み取りテスト
    println!("Step 1: Testing MTIME access...");
    let mtime_value = read_mtime();
    println_number!("Current MTIME: ", mtime_value);

    if mtime_value > 0 && mtime_value < u64::MAX {
        println!("✓ Timer hardware accessible");
    } else {
        println!("✗ Timer hardware issue");
        unsafe {
            TIMER_ERRORS += 1;
        }
        return;
    }

    // Step 2: MTIMECMP初期化（非常に遠い未来に設定）
    println!("Step 2: Initializing MTIMECMP...");
    let safe_future = mtime_value + (TIMER_FREQ * 3600); // 1時間後
    write_mtimecmp(safe_future);

    let readback = read_mtimecmp();
    if readback == safe_future {
        println!("✓ MTIMECMP initialized successfully");
        unsafe {
            LAST_MTIMECMP_VALUE = safe_future;
        }
    } else {
        println!("✗ MTIMECMP initialization failed");
        unsafe {
            TIMER_ERRORS += 1;
        }
    }

    // Step 3: 統計の初期化
    unsafe {
        TICKS = 0;
        TIMER_INTERRUPT_COUNT = 0;
    }

    println!("✓ Timer system initialized");
}

/// 安全なMTIME読み取り
pub fn read_mtime() -> u64 {
    unsafe { core::ptr::read_volatile(MTIME_ADDR) }
}

/// 安全なMTIMECMP書き込み
pub fn write_mtimecmp(value: u64) {
    unsafe {
        core::ptr::write_volatile(MTIMECMP_ADDR, value);
        LAST_MTIMECMP_VALUE = value;
    }
}

/// 安全なMTIMECMP読み取り
pub fn read_mtimecmp() -> u64 {
    unsafe { core::ptr::read_volatile(MTIMECMP_ADDR) }
}

/// タイマ割り込みハンドラ（trap.rsから呼ばれる）
pub fn handle_timer_interrupt() {
    unsafe {
        TIMER_INTERRUPT_COUNT += 1;
        TICKS += 1;
    }

    // 次のタイマ割り込みを設定（10秒間隔）
    let current_time = read_mtime();
    let next_interrupt = current_time + (TIMER_FREQ * 10); // 10秒後
    write_mtimecmp(next_interrupt);

    // 簡潔な出力（デバッグ用）
    unsafe {
        // Tick番号を表示（1tickごと）
        if TICKS % 1 == 0 {
            // 毎回表示
            core::ptr::write_volatile(UART0, b'T');
            core::ptr::write_volatile(UART0, b'K');

            // tick数の下位4ビットを16進表示
            let tick_low = TICKS & 0xF;
            let hex_char = if tick_low < 10 {
                b'0' + tick_low as u8
            } else {
                b'a' + (tick_low - 10) as u8
            };
            core::ptr::write_volatile(UART0, hex_char);
            core::ptr::write_volatile(UART0, b'\n');
        }
    }
}

/// ミリ秒単位での現在時刻を取得
pub fn get_time_ms() -> u64 {
    read_mtime() / (TIMER_FREQ / 1000)
}

/// 現在のtick数を取得
pub fn get_ticks() -> u64 {
    unsafe { TICKS }
}

/// タイマ割り込み統計を取得
pub fn get_timer_statistics() -> (u64, u64, u64) {
    unsafe { (TIMER_INTERRUPT_COUNT, TICKS, TIMER_ERRORS) }
}

/// 安全な遅延関数
pub fn safe_delay_test() {
    println!("=== SAFE DELAY TEST ===");

    let start_time = read_mtime();
    println_number!("Start time: ", start_time);

    // 1秒間の遅延をテスト
    let delay_ticks = TIMER_FREQ; // 1秒
    let target_time = start_time + delay_ticks;

    println_number!("Target time: ", target_time);
    println_number!("Delay duration (ticks): ", delay_ticks);

    let mut loop_count = 0;
    let max_loops = 1000000; // 安全上限

    while loop_count < max_loops {
        let current_time = read_mtime();

        if current_time >= target_time {
            let actual_elapsed = current_time - start_time;
            println_number!("Actual elapsed: ", actual_elapsed);
            println_number!("Loop count: ", loop_count);
            println!("✓ Delay test completed successfully");
            return;
        }

        loop_count += 1;

        // CPU負荷軽減
        if loop_count % 10000 == 0 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    println!("⚠ Delay test reached safety limit");
    unsafe {
        TIMER_ERRORS += 1;
    }
}

/// タイマアドレス情報のデバッグ表示
pub fn debug_timer_addresses() {
    println!("=== TIMER ADDRESS DEBUG ===");
    println_hex!("MTIME address: ", MTIME_ADDR as usize);
    println_hex!("MTIMECMP address: ", MTIMECMP_ADDR as usize);

    let mtime_val = read_mtime();
    let mtimecmp_val = read_mtimecmp();

    println_number!("Current MTIME: ", mtime_val);
    println_number!("Current MTIMECMP: ", mtimecmp_val);

    if mtimecmp_val > mtime_val {
        let remaining = mtimecmp_val - mtime_val;
        println_number!("Time until next interrupt: ", remaining);
        let seconds = remaining / TIMER_FREQ;
        println_number!("Seconds until next interrupt: ", seconds);
    } else {
        println!("⚠ MTIMECMP is in the past!");
    }
}

/// メモリ情報の表示（統合システム版）
pub fn show_memory_info() {
    println!("=== MEMORY LAYOUT (INTEGRATED SYSTEM) ===");
    println_hex!("UART0: ", crate::UART0 as usize);
    println_hex!("MTIME: ", MTIME_ADDR as usize);
    println_hex!("MTIMECMP: ", MTIMECMP_ADDR as usize);

    // CLINT範囲の表示
    let clint_base = 0x2000000;
    let clint_end = clint_base + 0x10000;
    println_hex!("CLINT range: ", clint_base);
    print!(" to ");
    print_hex!(clint_end);
    println!();
}

/// タイマ割り込み有効化の準備（段階的）
pub fn prepare_timer_interrupts() {
    println!("=== PREPARING TIMER INTERRUPTS ===");

    // 現在の状態確認
    let current_time = read_mtime();
    let current_mtimecmp = read_mtimecmp();

    println_number!("Current MTIME: ", current_time);
    println_number!("Current MTIMECMP: ", current_mtimecmp);

    // 非常に安全な間隔で最初の割り込みを設定
    let safe_interval = TIMER_FREQ * 30; // 30秒間隔
    let first_interrupt = current_time + safe_interval;

    println!("Setting first timer interrupt...");
    write_mtimecmp(first_interrupt);

    println_number!("First interrupt at: ", first_interrupt);
    println_number!("Interval (seconds): ", safe_interval / TIMER_FREQ);

    println!("✓ Timer interrupts prepared");
}

/// タイマ割り込み統計の表示
pub fn display_timer_statistics() {
    println!("=== TIMER STATISTICS ===");

    let (interrupts, ticks, errors) = get_timer_statistics();
    let current_mtime = read_mtime();
    let current_mtimecmp = read_mtimecmp();

    println_number!("Timer interrupts: ", interrupts);
    println_number!("Ticks: ", ticks);
    println_number!("Errors: ", errors);
    println_number!("Current MTIME: ", current_mtime);
    println_number!("Current MTIMECMP: ", current_mtimecmp);

    if errors > 0 {
        let error_rate = (errors * 100) / if interrupts > 0 { interrupts } else { 1 };
        println_number!("Error rate (%): ", error_rate);
    }

    // 次の割り込みまでの時間
    if current_mtimecmp > current_mtime {
        let remaining = current_mtimecmp - current_mtime;
        let seconds = remaining / TIMER_FREQ;
        println_number!("Next interrupt in (sec): ", seconds);
    }
}

/// 短期間のタイマ割り込みテスト
pub fn test_short_timer_interrupt() -> Result<(), &'static str> {
    println!("=== SHORT TIMER INTERRUPT TEST ===");

    let current_time = read_mtime();
    let test_interval = TIMER_FREQ * 5; // 5秒間隔
    let test_target = current_time + test_interval;

    println!("Setting 5-second timer interrupt...");
    write_mtimecmp(test_target);

    println_number!("Test target: ", test_target);
    println!("Waiting for interrupt...");

    // 最大10秒待機
    let max_wait_time = current_time + (TIMER_FREQ * 10);
    let mut checks = 0;

    while read_mtime() < max_wait_time {
        let now = read_mtime();

        if now >= test_target {
            println!("✓ Timer interrupt should have fired");
            // タイマを安全な状態に戻す
            let safe_future = now + (TIMER_FREQ * 3600);
            write_mtimecmp(safe_future);
            return Ok(());
        }

        checks += 1;
        if checks % 100000 == 0 {
            // 進捗表示
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    // タイマを安全な状態に戻す
    let safe_future = read_mtime() + (TIMER_FREQ * 3600);
    write_mtimecmp(safe_future);

    Err("Timer interrupt test timeout")
}
