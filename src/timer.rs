// RISC-V タイマ割り込み実装
// QEMU virt machineのメモリマップに基づく

use crate::{arch::csr, println, println_number, UART0};

// QEMU virt machine のタイマアドレス（修正版）
const MTIME_ADDR: *mut u64 = 0x200BFF8 as *mut u64; // 現在時刻
const MTIMECMP_ADDR: *mut u64 = 0x2004000 as *mut u64; // 比較値

// タイマ頻度（QEMU virtでは通常10MHz）
const TIMER_FREQ: u64 = 10_000_000;

// グローバルなtickカウンタ
static mut TICKS: u64 = 0;

// 次のタイマ割り込み時刻
static mut NEXT_TIMER: u64 = 0;

/// タイマシステムを初期化（段階的テスト版）
pub fn init_timer() {
    println!("Initializing timer with correct addresses...");

    // Step 1: 基本的な読み取りテスト
    println!("Step 1: Testing timer register access...");
    let mtime_value = read_mtime();
    println_number!("mtime: ", mtime_value);

    if mtime_value > 0 && mtime_value < u64::MAX {
        println!("SUCCESS: Timer appears to be working!");

        // Step 2: mtimecmpに安全な値を書き込み
        println!("Step 2: Testing mtimecmp write...");
        let safe_future_time = mtime_value + 1000000; // 現在時刻より十分後
        write_mtimecmp(safe_future_time);
        println_number!("Set mtimecmp to: ", safe_future_time);

        println!("Timer hardware access successful!");
    } else {
        println!("Timer read failed - keeping interrupts disabled");
    }
}

/// mtimeレジスタから現在時刻を読み取り（安全版）
pub fn read_mtime() -> u64 {
    // まず安全性チェック用の簡単なテスト
    unsafe {
        // QEMUのCLINTアドレス範囲内かチェック
        let addr = MTIME_ADDR as usize;
        if addr < 0x2000000 || addr > 0x200ffff {
            // 明らかに範囲外の場合は0を返す
            return 0;
        }

        // 実際の読み取り試行
        core::ptr::read_volatile(MTIME_ADDR)
    }
}

/// mtimecmpレジスタに比較値を書き込み（安全版）
pub fn write_mtimecmp(value: u64) {
    unsafe {
        // QEMUのCLINTアドレス範囲内かチェック
        let addr = MTIMECMP_ADDR as usize;
        if addr < 0x2000000 || addr > 0x200ffff {
            // 明らかに範囲外の場合は何もしない
            return;
        }

        core::ptr::write_volatile(MTIMECMP_ADDR, value);
    }
}

/// 段階的にタイマ割り込みを有効化する関数（安全版）
pub fn enable_timer_interrupts() {
    println!("Attempting to enable timer interrupts...");

    let current_time = read_mtime();
    if current_time == 0 {
        println!("ERROR: Timer not working - interrupts not enabled");
        return;
    }

    // 10秒後に最初の割り込みを設定（非常に安全な間隔）
    let interval = TIMER_FREQ * 10; // 10秒間隔
    unsafe {
        NEXT_TIMER = current_time + interval;
        write_mtimecmp(NEXT_TIMER);
    }

    println!("Step 1: Set mtimecmp to far future");
    println_number!("Current time: ", current_time);
    println_number!("Target time: ", unsafe { NEXT_TIMER });

    // CSR操作を段階的に実行
    println!("Step 2: Enabling MTIE...");
    unsafe {
        csr::enable_machine_timer_interrupt();
    }

    // MIE確認
    let mie = csr::read_mie();
    println_hex!("mie after MTIE enable: ", mie);

    println!("Step 3: Enabling global interrupts...");
    unsafe {
        csr::enable_global_interrupts();
    }

    // 最終確認
    let mstatus = csr::read_mstatus();
    let final_mie = csr::read_mie();
    println_hex!("Final mstatus: ", mstatus);
    println_hex!("Final mie: ", final_mie);

    println!("Timer interrupts enabled (10 second interval)!");
}

/// タイマテスト用の安全な遅延関数（修正版）
pub fn safe_delay_test() {
    println!("Testing timer-based delay...");

    let start_time = read_mtime();

    // より短い間隔でテスト（10ms = TIMER_FREQ / 100）
    let short_interval = TIMER_FREQ / 100; // 10ms
    let target_time = start_time + short_interval;

    println_number!("Start time: ", start_time);
    println_number!("Target time: ", target_time);
    println_number!("Interval (10ms): ", short_interval);

    // より多くのループ回数で安全性確保
    let mut count = 0;
    let max_loops = 100000; // 10万回まで

    while count < max_loops {
        let current_time = read_mtime();

        if current_time >= target_time {
            // 目標時刻に到達
            let end_time = current_time;
            println_number!("End time: ", end_time);
            println_number!("Elapsed ticks: ", end_time - start_time);
            println_number!("Loop count: ", count);
            println!("Delay test PASSED");
            return;
        }

        count += 1;

        // CPU負荷軽減のため、たまにnop
        if count % 1000 == 0 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    // タイムアウト
    let end_time = read_mtime();
    println_number!("End time: ", end_time);
    println_number!("Elapsed ticks: ", end_time - start_time);
    println_number!("Max loops reached: ", max_loops);
    println!("Delay test TIMEOUT (safe exit)");
}

/// ミリ秒単位での現在時刻を取得
pub fn get_time_ms() -> u64 {
    read_mtime() / (TIMER_FREQ / 1000)
}

/// 非常に安全なタイマ割り込みテスト
pub fn test_timer_interrupts_safe() {
    println!("=== SAFE TIMER INTERRUPT TEST ===");

    // Step 1: 現在の状態確認
    println!("Step 1: Current state check");
    let current_time = read_mtime();
    let mstatus = csr::read_mstatus();
    let mie = csr::read_mie();

    println_number!("Current mtime: ", current_time);
    println_hex!("Current mstatus: ", mstatus);
    println_hex!("Current mie: ", mie);

    // Step 2: 非常に遠い未来にmtimecmpを設定
    println!("Step 2: Setting mtimecmp to very far future");
    let very_far_future = current_time + (TIMER_FREQ * 3600); // 1時間後
    write_mtimecmp(very_far_future);
    // println_number!("Set mtimecmp to: ", very_far_future);

    // Step 3: 現在のmtimecmp値を確認
    unsafe {
        let readback = core::ptr::read_volatile(MTIMECMP_ADDR);
        // println_number!("mtimecmp readback: ", readback);
        if readback != very_far_future {
            println!("ERROR: mtimecmp write failed!");
            return;
        }
    }

    println!("All checks passed - timer hardware is safe");
    println!("Skipping interrupt enable for now");
}

/// タイマ割り込みハンドラ
pub fn handle_timer_interrupt() {
    unsafe {
        TICKS += 1;

        // 次の割り込み時刻を設定（10秒後）
        let interval = TIMER_FREQ * 10; // 10秒間隔
        NEXT_TIMER += interval;
        write_mtimecmp(NEXT_TIMER);

        // 1tickごとに出力（10秒ごと）
        println_number!("TICK: ", TICKS);
    }
}
pub fn get_ticks() -> u64 {
    unsafe { TICKS }
}

/// 複数のアドレスでmtimeを試す安全なテスト（修正版）
pub fn debug_timer_addresses() {
    println!("Testing timer addresses with correct values...");

    // 正しいアドレスのテスト
    println!("Testing known correct addresses:");

    // mtime test
    println_hex!("Testing mtime at: ", MTIME_ADDR as usize);
    let mtime_val = read_mtime();
    println_number!("  mtime value: ", mtime_val);

    // mtimecmp test
    println_hex!("Testing mtimecmp at: ", MTIMECMP_ADDR as usize);
    // mtimecmpは読み取り専用ではないので、現在の値を確認
    unsafe {
        let mtimecmp_val = core::ptr::read_volatile(MTIMECMP_ADDR);
        println_number!("  mtimecmp value: ", mtimecmp_val);
    }

    println!("Timer address test complete");
}

/// 現在のメモリマップ情報を表示（修正版）
pub fn show_memory_info() {
    println!("Memory layout info:");
    println_hex!("UART0: ", crate::UART0 as usize);
    println_hex!("MTIME: ", MTIME_ADDR as usize);
    println_hex!("MTIMECMP: ", MTIMECMP_ADDR as usize);
}
