#![no_std]
#![no_main]

#[macro_use]
mod console;

mod arch;
mod interrupt;
mod msip_debug; // MSIP安全性検証モジュールを追加
mod trap; // trap機能を追加 // ソフトウェア割り込みモジュールを追加

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

    // Phase 8: ソフトウェア割り込み基本実装（簡素版）
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
    match interrupt::yield_cpu() {
        Ok(()) => println!("✓ Single yield successful"),
        Err(e) => {
            print!("✗ Single yield failed: ");
            println!(e);
        }
    }

    // Phase 10: システム安定性の最終確認
    println!("\n=== PHASE 10: FINAL STABILITY CHECK ===");
    println!("Running final stability test...");

    let mut final_counter = 0u64;

    loop {
        final_counter = final_counter.wrapping_add(1);

        if final_counter % 10000000 == 0 {
            println_number!("Final stable count: ", final_counter);

            // 統計情報表示
            if final_counter % 50000000 == 0 {
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

    println_number!("mstatus.MIE: ", mie_bit);
    println_number!("mstatus.MPIE: ", mpie_bit);
    println_number!("mstatus.MPP: ", mpp_bits);

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
    println!("PANIC in safe trap mode!");
    if let Some(location) = info.location() {
        print!("Location: ");
        print!(location.file());
        print!(":");
        print_number!(location.line());
        println!();
    }

    // パニック時のCSR状態
    let mstatus = arch::csr::read_mstatus();
    let mcause = arch::csr::read_mcause();
    let mepc = arch::csr::read_mepc();
    let mtvec = arch::csr::read_mtvec();

    println!("Panic CSR state:");
    println_hex!("  mstatus: ", mstatus);
    println_hex!("  mcause: ", mcause);
    println_hex!("  mepc: ", mepc);
    println_hex!("  mtvec: ", mtvec);

    loop {
        unsafe {
            core::arch::asm!("nop");
        }
    }
}
