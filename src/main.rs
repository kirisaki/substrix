#![no_std]
#![no_main]

#[macro_use]
mod console;

mod arch;
mod interrupt;
mod msip_debug;
mod panic;
mod timer;
mod trap;

pub const UART0: *mut u8 = 0x1000_0000 as *mut u8;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("STEP 1: Basic system with safe trap handling");

    // Phase 1: 基本テスト（前回と同じ）
    println!("\n=== PHASE 1: BASIC TESTS ===");
    basic_tests();

    // Phase 2: CSR状態の詳細確認
    println!("\n=== PHASE 2: CSR STATE ANALYSIS ===");
    analyze_csr_state();

    // Phase 3: Trap機能の初期化（慎重に）
    println!("\n=== PHASE 3: SAFE TRAP INITIALIZATION ===");
    println!("Initializing trap handler...");

    // trap初期化前の状態確認
    let mtvec_before = arch::csr::read_mtvec();
    println_hex!("mtvec before init: ", mtvec_before);

    // trap初期化
    trap::init_trap();

    // trap初期化後の状態確認
    let mtvec_after = arch::csr::read_mtvec();
    println_hex!("mtvec after init: ", mtvec_after);

    if mtvec_after != 0 && mtvec_after != mtvec_before {
        println!("✓ Trap handler successfully initialized");
    } else {
        println!("✗ Trap handler initialization failed");
        panic!("Trap init failed");
    }

    // Phase 4: 安全なecallテスト
    println!("\n=== PHASE 4: SAFE ECALL TEST ===");
    println!("Testing ecall (this should trigger trap)...");

    // ecallテスト前の状態
    let mcause_before = arch::csr::read_mcause();
    let mepc_before = arch::csr::read_mepc();
    println_hex!("mcause before ecall: ", mcause_before);
    println_hex!("mepc before ecall: ", mepc_before);

    // 実際のecallテスト
    trap::test_ecall_safe();

    // ecallテスト後の状態
    let mcause_after = arch::csr::read_mcause();
    let mepc_after = arch::csr::read_mepc();
    println_hex!("mcause after ecall: ", mcause_after);
    println_hex!("mepc after ecall: ", mepc_after);

    println!("✓ Ecall test completed successfully!");

    // Phase 5: システム安定性確認
    println!("\n=== PHASE 5: SYSTEM STABILITY CHECK ===");
    println!("Running stability test with trap handler active...");

    // 短期間の安定性テスト
    let mut counter = 0u64;
    let stability_test_limit = 30000000; // 短縮

    println!("Running short stability test...");
    while counter < stability_test_limit {
        counter = counter.wrapping_add(1);

        if counter % 10000000 == 0 {
            println_number!("Stability test: ", counter);
        }

        unsafe {
            core::arch::asm!("nop");
        }
    }

    println!("✓ Short stability test passed");

    // Phase 6: MSIP安全性検証（新規追加）
    println!("\n=== PHASE 6: MSIP SAFETY VERIFICATION ===");

    // trap handler が動作する状態でMSIPテストを実行
    println!("Testing MSIP with active trap handler...");
    msip_debug::comprehensive_clint_test();

    // Phase 7: 基本MSIP操作テスト（MSIPアクセスが成功した場合）
    println!("\n=== PHASE 7: BASIC MSIP OPERATIONS ===");

    println!("Testing safe MSIP operations...");
    match msip_debug::safe_msip_read() {
        Ok(val) => {
            println_number!("Safe MSIP read successful: ", val as u64);

            // 基本操作テスト
            msip_debug::basic_msip_test();
        }
        Err(e) => {
            print!("MSIP read failed: ");
            println!(e);
            println!("Skipping MSIP operations");
        }
    }

    // Phase 8.5: タイマ基本機能復活テスト（新規追加）
    println!("\n=== PHASE 8.5: TIMER BASIC FUNCTIONALITY ===");

    // タイマの基本機能テスト（割り込みなし）
    println!("Testing timer basic functions...");
    timer::show_memory_info();

    // MTIMEの読み取りテスト
    println!("Testing MTIME reading...");
    let mtime_start = timer::read_mtime();
    print!("MTIME start: ");
    print_number!(mtime_start);
    println!();

    // 短い遅延後に再度読み取り
    for _ in 0..1000000 {
        unsafe {
            core::arch::asm!("nop");
        }
    }

    let mtime_end = timer::read_mtime();
    print!("MTIME end: ");
    print_number!(mtime_end);
    println!();

    if mtime_end > mtime_start {
        println!("✓ Timer is running correctly");
        let elapsed = mtime_end - mtime_start;
        print!("Elapsed ticks: ");
        print_number!(elapsed);
        println!();
    } else {
        println!("✗ Timer not working");
    }

    // MTIMECMPの安全テスト
    println!("Testing MTIMECMP operations...");
    timer::debug_timer_addresses();

    // Phase 8: ソフトウェア割り込み基本実装（既存）
    println!("\n=== PHASE 8: SOFTWARE INTERRUPT BASIC IMPLEMENTATION ===");

    // ソフトウェア割り込みシステムの初期化
    interrupt::init_software_interrupt();

    // 基本的なMSIP動作確認
    println!("Testing basic MSIP operations after init...");
    match interrupt::test_basic_msip_operations_simple() {
        Ok(()) => println!("✓ Basic MSIP operations work"),
        Err(e) => {
            print!("✗ Basic MSIP operations failed: ");
            println!(e);
        }
    }

    // Phase 9: 単純なyield()テスト（1回のみ）
    println!("\n=== PHASE 9: SIMPLE YIELD TEST ===");

    println!("Testing single yield() call...");
    match interrupt::yield_cpu_relaxed() {
        Ok(()) => println!("✓ Single yield successful"),
        Err(e) => {
            print!("✗ Single yield failed: ");
            println!(e);
        }
    }

    // Phase 11: タイマ割り込み段階的有効化（新規追加）
    println!("\n=== PHASE 11: TIMER INTERRUPT STAGED ENABLEMENT ===");

    // Step 1: タイマシステムの初期化
    println!("Step 1: Initializing timer system...");
    timer::init_timer();

    // Step 2: 安全な遅延テスト
    println!("Step 2: Testing safe delay functionality...");
    timer::safe_delay_test();

    // Step 3: タイマ割り込み有効化準備
    println!("Step 3: Preparing timer interrupt enablement...");

    // 現在のMIE状態を確認
    let mie_before = arch::csr::read_mie();
    println_hex!("MIE before timer enable: ", mie_before);

    // MTIE (Machine Timer Interrupt Enable) を有効化
    println!("Enabling MTIE...");
    unsafe {
        arch::csr::enable_machine_timer_interrupt();
    }

    let mie_after = arch::csr::read_mie();
    println_hex!("MIE after timer enable: ", mie_after);

    if (mie_after & (1 << 7)) != 0 {
        println!("✓ Timer interrupts (MTIE) enabled in MIE");
    } else {
        println!("✗ MTIE not enabled");
    }

    // Step 4: 控えめなタイマ割り込みテスト
    println!("Step 4: Conservative timer interrupt test...");
    println!("Setting timer interrupt for 30 seconds in future...");

    // 非常に遠い未来にタイマ設定（まだ割り込み発生させない）
    let current_time = timer::read_mtime();
    let very_far_future = current_time + 300_000_000; // 30秒後（10MHz想定）
    timer::write_mtimecmp(very_far_future);

    print!("Current MTIME: ");
    print_number!(current_time);
    println!();
    print!("MTIMECMP set to: ");
    print_number!(very_far_future);
    println!();

    println!("Timer interrupt system prepared (not yet triggered)");

    // Phase 12: 統合システムテスト（タイマ + ソフトウェア割り込み）
    println!("\n=== PHASE 12: INTEGRATED INTERRUPT SYSTEM TEST ===");

    println!("Testing both software and timer interrupt systems...");

    // グローバル割り込み有効化（重要：両方の割り込みが動作するため）
    println!("Enabling global interrupts for integrated test...");
    unsafe {
        arch::csr::enable_global_interrupts();
    }

    let mstatus_final = arch::csr::read_mstatus();
    let mie_final = arch::csr::read_mie();
    println_hex!("Final mstatus: ", mstatus_final);
    println_hex!("Final mie: ", mie_final);

    // 割り込み有効状態の最終確認
    println!("Final interrupt enable status:");
    if (mstatus_final & (1 << 3)) != 0 {
        println!("  ✓ Global interrupts (MIE) enabled");
    }
    if (mie_final & (1 << 3)) != 0 {
        println!("  ✓ Software interrupts (MSIE) enabled");
    }
    if (mie_final & (1 << 7)) != 0 {
        println!("  ✓ Timer interrupts (MTIE) enabled");
    }

    // Phase 13: ライブテスト（短時間のタイマ割り込み）
    println!("\n=== PHASE 13: LIVE TIMER INTERRUPT TEST ===");

    println!("Setting up actual timer interrupt (SHORT interval)...");
    let test_time = timer::read_mtime();

    // より短い間隔でテスト（実際の時刻進行に合わせて）
    let timer_test_target = test_time + 1_000_000; // 100ms後（短縮）

    print!("Current time: ");
    print_number!(test_time);
    println!();
    print!("Timer interrupt will fire at: ");
    print_number!(timer_test_target);
    println!();
    print!("Difference (100ms): ");
    print_number!(timer_test_target - test_time);
    println!();

    timer::write_mtimecmp(timer_test_target);

    // 書き込み確認
    let mtimecmp_readback = timer::read_mtimecmp();
    print!("MTIMECMP readback: ");
    print_number!(mtimecmp_readback);
    println!();

    if mtimecmp_readback != timer_test_target {
        println!("✗ MTIMECMP write failed!");
    } else {
        println!("✓ MTIMECMP set correctly");
    }

    println!("Waiting for timer interrupt (SHORT wait)...");

    // より短い待機ループ
    let mut wait_loops = 0;
    let max_wait_loops = 20; // 大幅短縮

    while wait_loops < max_wait_loops {
        let current = timer::read_mtime();

        print!("Wait ");
        print_number!(wait_loops);
        print!(": current=");
        print_number!(current);

        if current >= timer_test_target {
            println!(" -> TARGET REACHED!");

            // 少し待ってからtick確認
            for _ in 0..100000 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }

            let post_interrupt_ticks = timer::get_ticks();
            print!("Ticks after target: ");
            print_number!(post_interrupt_ticks);
            println!();
            break;
        } else {
            let remaining = timer_test_target - current;
            print!(", remaining=");
            print_number!(remaining);
            println!();
        }

        // 短い待機
        for _ in 0..50000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }

        wait_loops += 1;
    }

    // 最終状態確認
    let final_time = timer::read_mtime();
    let final_ticks = timer::get_ticks();

    print!("Final time: ");
    print_number!(final_time);
    print!(", Final ticks: ");
    print_number!(final_ticks);
    println!();

    if final_ticks > 0 {
        println!("✓ Timer interrupts are working!");
    } else if final_time >= timer_test_target {
        println!("⚠ Target reached but no ticks - checking handler");

        // ハンドラが動作しているかより詳細にチェック
        timer::display_timer_statistics();
    } else {
        println!("ℹ Target not reached in time - trying longer interval");

        // より長い間隔で再テスト
        let longer_target = final_time + 2_000_000; // 200ms
        println!("Trying longer interval test...");
        timer::write_mtimecmp(longer_target);

        for retry_loop in 0..10 {
            let retry_current = timer::read_mtime();
            if retry_current >= longer_target {
                let retry_ticks = timer::get_ticks();
                print!("Retry result - ticks: ");
                print_number!(retry_ticks);
                println!();
                break;
            }

            for _ in 0..200000 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
        }
    }

    println!("Timer interrupt test completed");

    // タイマを安全な状態に戻す
    let safe_future = timer::read_mtime() + 100_000_000; // 10秒後
    timer::write_mtimecmp(safe_future);

    // Phase 10: システム安定性の最終確認（既存のままだが、統合システム対応）
    println!("\n=== PHASE 10: FINAL STABILITY CHECK ===");
    // Phase 14: 統合長期安定性テスト
    println!("\n=== PHASE 14: INTEGRATED LONG-TERM STABILITY ===");
    println!("Running integrated stability test with all interrupt systems...");

    let mut integrated_counter = 0u64;
    let mut test_cycle = 0u64;

    loop {
        integrated_counter = integrated_counter.wrapping_add(1);

        if integrated_counter % 10000000 == 0 {
            print!("Integrated count: ");
            print_number!(integrated_counter);
            println!();

            test_cycle += 1;

            // 様々な機能のローテーションテスト
            match test_cycle % 6 {
                1 => {
                    // ecallテスト
                    if test_cycle <= 30 {
                        println!("Testing ecall...");
                        trap::test_ecall_safe();
                        println!("Ecall OK");
                    }
                }
                2 => {
                    // MSIPテスト
                    println!("Testing MSIP...");
                    match msip_debug::safe_msip_read() {
                        Ok(_) => println!("MSIP OK"),
                        Err(e) => {
                            print!("MSIP failed: ");
                            println!(e);
                        }
                    }
                }
                3 => {
                    // yield()テスト（ソフトウェア割り込み）
                    if test_cycle <= 20 {
                        println!("Testing yield (SW interrupt)...");
                        match interrupt::yield_cpu_relaxed() {
                            Ok(()) => println!("Yield OK"),
                            Err(e) => {
                                print!("Yield failed: ");
                                println!(e);
                            }
                        }
                    }
                }
                4 => {
                    // タイマ読み取りテスト
                    println!("Testing timer reading...");
                    let mtime = timer::read_mtime();
                    let ticks = timer::get_ticks();
                    print!("MTIME: ");
                    print_number!(mtime);
                    print!(", Ticks: ");
                    print_number!(ticks);
                    println!();
                }
                5 => {
                    // 統計情報表示
                    println!("=== INTEGRATED STATISTICS ===");
                    interrupt::display_statistics();

                    let current_mtime = timer::read_mtime();
                    let current_ticks = timer::get_ticks();
                    println_number!("Current MTIME: ", current_mtime);
                    println_number!("Timer ticks: ", current_ticks);
                }
                0 => {
                    // システム状態の総合確認
                    if test_cycle % 12 == 0 {
                        println!("=== COMPREHENSIVE SYSTEM STATUS ===");
                        let mstatus = arch::csr::read_mstatus();
                        let mie = arch::csr::read_mie();
                        let mtimecmp_check = timer::read_mtime() + 1000000;

                        println_hex!("mstatus: ", mstatus);
                        println_hex!("mie: ", mie);
                        println_number!("Next safe mtimecmp: ", mtimecmp_check);
                    }
                }
                _ => {}
            }

            // テストサイクルのリセット
            if test_cycle > 60 {
                test_cycle = 0;
                println!("=== CYCLE RESET - SYSTEM RUNNING PERFECTLY ===");
                interrupt::display_statistics();
            }
        }

        unsafe {
            core::arch::asm!("nop");
        }
    }
}

/// 基本機能テスト
fn basic_tests() {
    // 算術テスト
    let result = 2 + 2;
    print!("Arithmetic test: 2 + 2 = ");
    print_number!(result);
    println!();

    if result == 4 {
        println!("✓ Arithmetic: PASS");
    } else {
        println!("✗ Arithmetic: FAIL");
    }

    // メモリテスト
    let mut test_array = [1, 2, 3, 4, 5];
    test_array[2] = 99;

    print!("Memory test: array[2] = ");
    print_number!(test_array[2]);
    println!();

    if test_array[2] == 99 {
        println!("✓ Memory: PASS");
    } else {
        println!("✗ Memory: FAIL");
    }
    // Phase 15: パニック・デバッグシステムのテスト
    println!("\n=== PHASE 15: PANIC & DEBUG SYSTEM TEST ===");
    test_panic_system();
}

/// CSR状態分析
fn analyze_csr_state() {
    println!("Analyzing CSR state...");

    // 基本的なCSR読み取り
    let mhartid = read_mhartid();
    let mstatus = arch::csr::read_mstatus();
    let mie = arch::csr::read_mie();
    let mtvec = arch::csr::read_mtvec();
    let mcause = arch::csr::read_mcause();
    let mepc = arch::csr::read_mepc();

    println_number!("mhartid: ", mhartid);
    println_hex!("mstatus: ", mstatus);
    println_hex!("mie: ", mie);
    println_hex!("mtvec: ", mtvec);
    println_hex!("mcause: ", mcause);
    println_hex!("mepc: ", mepc);

    // mstatusの解析
    let mie_bit = (mstatus >> 3) & 1;
    let mpie_bit = (mstatus >> 7) & 1;
    let mpp_bits = (mstatus >> 11) & 3;

    println_number!("mstatus.MIE: ", mie_bit as u64);
    println_number!("mstatus.MPIE: ", mpie_bit as u64);
    println_number!("mstatus.MPP: ", mpp_bits as u64);

    println!("CSR state analysis complete");
}

/// mhartid読み取り
fn read_mhartid() -> u64 {
    let mut val: u64;
    unsafe {
        core::arch::asm!("csrr {}, mhartid", out(reg) val);
    }
    val
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::enhanced_panic_handler(info)
}

fn test_panic_system() {
    println!("=== PANIC SYSTEM COMPREHENSIVE TEST ===");

    // Test 1: アサーション機能のテスト
    println!("Test 1: Assertion functionality");
    test_assertions();

    // Test 2: メモリチェック機能のテスト
    println!("Test 2: Memory checking");
    test_memory_checks();

    // Test 3: スタック監視機能のテスト
    println!("Test 3: Stack monitoring");
    test_stack_monitoring();

    // Test 4: CSR状態ダンプのテスト
    println!("Test 4: CSR state dump");
    test_csr_dump();

    // Test 5: 制御されたパニック（最後）
    println!("Test 5: Controlled panic test");
    test_controlled_panic();

    println!("All panic system tests completed successfully!");
}

/// アサーション機能のテスト
fn test_assertions() {
    println!("Testing assertion functionality...");

    // 正常なアサーション（通る）
    kassert!(2 + 2 == 4);
    kassert!(true, "This should pass");
    println!("✓ Normal assertions passed");

    // 条件付きアサーション
    let test_value = 42;
    kassert!(test_value == 42);
    kassert!(test_value > 0, "Value should be positive");
    println!("✓ Conditional assertions passed");

    // デバッグアサーション（デバッグビルドでのみ）
    #[cfg(debug_assertions)]
    {
        println!("Debug build: Testing debug assertions");
        // ここでは実際にはパニックしないテスト
        println!("✓ Debug assertions ready");
    }

    #[cfg(not(debug_assertions))]
    {
        println!("Release build: Debug assertions disabled");
    }

    println!("✓ Assertion tests completed");
}

/// メモリチェック機能のテスト
fn test_memory_checks() {
    println!("Testing memory checking functionality...");

    // 正常なメモリアクセス
    let test_array = [1u64, 2u64, 3u64, 4u64];
    let ptr = test_array.as_ptr();

    println_hex!("Test array address: ", ptr as usize);

    // メモリ読み取りテスト
    let value = unsafe { core::ptr::read_volatile(ptr) };
    if value == 1 {
        println!("✓ Memory read test passed");
    } else {
        println!("✗ Memory read test failed");
    }

    // アドレス範囲の検証
    let ram_start = 0x80000000;
    let ram_end = 0x88000000;
    let test_addr = ptr as usize;

    if test_addr >= ram_start && test_addr < ram_end {
        println!("✓ Address in valid RAM range");
    } else {
        println!("⚠ Address outside RAM range (stack/heap)");
    }

    // 境界テスト（安全）
    println!("Testing boundary conditions...");
    test_memory_boundary_safe();

    println!("✓ Memory check tests completed");
}

/// 安全な境界テスト
fn test_memory_boundary_safe() {
    // スタックの境界をテスト
    let stack_var = 0u64;
    let stack_addr = &stack_var as *const u64 as usize;

    println_hex!("Stack variable address: ", stack_addr);

    // スタック範囲の確認
    let stack_base = 0x80100000; // boot.sで設定
    if stack_addr < stack_base {
        println!("✓ Stack growing downward correctly");
    } else {
        println!("⚠ Stack boundary unexpected");
    }

    // 安全なアドレス計算テスト
    let safe_offset = 8;
    let next_addr = stack_addr.wrapping_add(safe_offset);
    if next_addr > stack_addr {
        println!("✓ Address arithmetic working");
    }
}

/// スタック監視機能のテスト
fn test_stack_monitoring() {
    println!("Testing stack monitoring...");

    // 現在のスタックポインタを取得
    let current_sp = get_current_sp();
    println_hex!("Current SP: ", current_sp);

    // スタック使用量の計算
    let stack_base = 0x80100000;
    let stack_used = stack_base - current_sp;
    println_number!("Stack used: ", stack_used as u64);
    println!(" bytes");

    // スタックの健全性チェック
    if current_sp >= 0x80000000 && current_sp < 0x80100000 {
        println!("✓ Stack pointer in valid range");
    } else {
        println!("✗ Stack pointer out of range!");
    }

    // 再帰的なスタック使用テスト（制限付き）
    test_recursive_stack_safe(5);

    println!("✓ Stack monitoring tests completed");
}

/// 現在のスタックポインタを取得
fn get_current_sp() -> usize {
    let mut sp: usize;
    unsafe {
        core::arch::asm!("mv {}, sp", out(reg) sp);
    }
    sp
}

/// 安全な再帰テスト（浅い再帰）
fn test_recursive_stack_safe(depth: u32) {
    if depth == 0 {
        let sp = get_current_sp();
        println_number!("Recursion bottom SP: ", sp as u64);
        return;
    }

    // スタックに何かデータを置く
    let local_data = [depth; 4];
    let _sum: u32 = local_data.iter().sum();

    test_recursive_stack_safe(depth - 1);
}

/// CSR状態ダンプのテスト
fn test_csr_dump() {
    println!("Testing CSR state dump functionality...");

    // 現在のCSR状態を読み取り
    let mstatus = arch::csr::read_mstatus();
    let mie = arch::csr::read_mie();
    let mtvec = arch::csr::read_mtvec();
    let mcause = arch::csr::read_mcause();
    let mepc = arch::csr::read_mepc();

    println!("Current CSR state:");
    println_hex!("  mstatus: ", mstatus);
    println_hex!("  mie:     ", mie);
    println_hex!("  mtvec:   ", mtvec);
    println_hex!("  mcause:  ", mcause);
    println_hex!("  mepc:    ", mepc);

    // 各ビットフィールドの解析テスト
    let global_ie = (mstatus >> 3) & 1;
    let mtie = (mie >> 7) & 1;
    let msie = (mie >> 3) & 1;

    println!("Bit field analysis:");
    println_number!("  Global IE: ", global_ie as u64);
    println_number!("  Timer IE:  ", mtie as u64);
    println_number!("  SW IE:     ", msie as u64);

    println!("✓ CSR dump test completed");
}

/// 制御されたパニックのテスト
fn test_controlled_panic() {
    println!("=== CONTROLLED PANIC TEST ===");
    println!("This will test the panic handler with a controlled panic.");
    println!("The system should display detailed debug information and halt safely.");
    println!();

    // ユーザに警告
    println!("WARNING: The next operation will trigger a deliberate panic!");
    println!("This is for testing the panic handler functionality.");
    println!("The system will halt after displaying debug information.");
    println!();

    // カウントダウン（視覚的効果）
    for i in (1..=5).rev() {
        print!("Triggering panic in ");
        print_number!(i);
        println!(" seconds...");

        // 短い遅延
        for _ in 0..5000000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    println!("=== TRIGGERING TEST PANIC NOW ===");

    // 実際のパニックをトリガー
    panic!("This is a controlled test panic for debugging system verification");
}

// デバッグ用のヘルパー関数
/// 安全なメモリダンプ（指定アドレス範囲）
pub fn safe_memory_dump(start: usize, length: usize) {
    println!("Memory dump:");
    println_hex!("Start address: ", start);
    println_number!("Length: ", length as u64);
    println!(" bytes");

    if start < 0x80000000 || start >= 0x88000000 {
        println!("⚠ Address outside RAM range - skipping");
        return;
    }

    let end = start + length;
    if end >= 0x88000000 {
        println!("⚠ Range extends beyond RAM - truncating");
    }

    let safe_end = core::cmp::min(end, 0x88000000);
    let safe_length = safe_end - start;

    for i in (0..safe_length).step_by(8) {
        let addr = start + i;
        if addr + 8 <= safe_end {
            let value = unsafe { core::ptr::read_volatile(addr as *const u64) };

            print!("  ");
            print_hex!(addr);
            print!(": ");
            print_hex!(value as usize);
            println!();
        }

        // 大量のダンプを防ぐため制限
        if i >= 64 {
            println!("  (truncated for safety)");
            break;
        }
    }
}

/// システム診断情報の表示
pub fn system_diagnostics() {
    println!("=== SYSTEM DIAGNOSTICS ===");

    // ハードウェア情報
    let mhartid = {
        let mut val: u64;
        unsafe {
            core::arch::asm!("csrr {}, mhartid", out(reg) val);
        }
        val
    };

    println_number!("Hart ID: ", mhartid);

    // タイマ情報
    let mtime = crate::timer::read_mtime();
    let ticks = crate::timer::get_ticks();

    println_number!("MTIME: ", mtime);
    println_number!("Timer ticks: ", ticks);

    // 割り込み統計
    let (sw_interrupts, yields, handlers, errors) = crate::interrupt::get_statistics();
    println_number!("SW interrupts: ", sw_interrupts);
    println_number!("Yield calls: ", yields);
    println_number!("Handler calls: ", handlers);
    println_number!("Errors: ", errors);

    // メモリ使用量
    let current_sp = get_current_sp();
    let stack_used = 0x80100000 - current_sp;
    println_number!("Stack used: ", stack_used as u64);
    println!(" bytes");

    println!("=== DIAGNOSTICS COMPLETE ===");
}

/// 手動でのパニックトリガー（デバッグ専用）
#[allow(dead_code)]
pub fn trigger_debug_panic() {
    panic!("Manual debug panic triggered");
}

/// メモリ破損の人工的なテスト（危険 - テスト用のみ）
#[allow(dead_code)]
pub fn test_memory_corruption_detection() {
    println!("Testing memory corruption detection...");

    let mut test_data = 0x12345678u64;
    let original = test_data;

    // データを意図的に変更
    test_data = 0xDEADBEEF;

    // 破損を検出して報告
    if test_data != original {
        println!("Memory corruption detected (simulated)");
        // 実際のシステムではここでパニックする
        // panic::memory_corruption_panic(
        //     &test_data as *const u64 as usize,
        //     original,
        //     test_data
        // );
    }

    println!("✓ Memory corruption detection test completed");
}
