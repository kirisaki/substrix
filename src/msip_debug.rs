// MSIP (Machine Software Interrupt Pending) 安全性検証
use crate::{println, println_hex, println_number, UART0};

// QEMU virt machine CLINT addresses
const CLINT_BASE: usize = 0x2000000;
const MSIP_BASE: usize = CLINT_BASE + 0x0; // MSIP for hart 0
const MTIMECMP_BASE: usize = CLINT_BASE + 0x4000; // MTIMECMP for hart 0
const MTIME_ADDR: usize = CLINT_BASE + 0xBFF8; // MTIME

/// CLINT領域の包括的安全性テスト
pub fn comprehensive_clint_test() {
    println!("=== COMPREHENSIVE CLINT SAFETY TEST ===");

    // Step 1: アドレス情報の表示
    display_clint_addresses();

    // Step 2: 既知の動作するMTIME読み取りテスト
    test_mtime_access();

    // Step 3: MTIMECMP領域の安全性テスト
    test_mtimecmp_access();

    // Step 4: MSIP領域の段階的アクセステスト
    test_msip_access_staged();

    println!("CLINT safety test completed");
}

/// CLINTアドレス情報の表示
fn display_clint_addresses() {
    println!("CLINT Address Layout:");
    println_hex!("  CLINT_BASE: ", CLINT_BASE);
    println_hex!("  MSIP (hart0): ", MSIP_BASE);
    println_hex!("  MTIMECMP (hart0): ", MTIMECMP_BASE);
    println_hex!("  MTIME: ", MTIME_ADDR);

    // アドレス範囲の確認
    let clint_end = CLINT_BASE + 0x10000;
    println_hex!("  CLINT range: ", CLINT_BASE);
    print!(" to ");
    print_hex!(clint_end);
    println!();
}

/// MTIME アクセステスト（既知の動作確認）
fn test_mtime_access() {
    println!("Testing MTIME access (known working)...");

    let mtime_ptr = MTIME_ADDR as *const u64;

    // 3回読み取って変化を確認
    for i in 1..=3 {
        let mtime_val = unsafe { core::ptr::read_volatile(mtime_ptr) };
        print!("  MTIME read #");
        print_number!(i);
        print!(": ");
        print_number!(mtime_val);
        println!();

        // 短い遅延
        for _ in 0..1000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    println!("✓ MTIME access successful");
}

/// MTIMECMP アクセステスト
fn test_mtimecmp_access() {
    println!("Testing MTIMECMP access...");

    let mtimecmp_ptr = MTIMECMP_BASE as *mut u64;

    // 現在値を読み取り
    let original_val = unsafe { core::ptr::read_volatile(mtimecmp_ptr) };
    print!("  MTIMECMP original: ");
    print_number!(original_val);
    println!();

    // 安全な値に設定（現在時刻より十分未来）
    let mtime_val = unsafe { core::ptr::read_volatile(MTIME_ADDR as *const u64) };
    let safe_future = mtime_val + 100000000; // 10秒後（10MHz想定）

    println!("  Setting MTIMECMP to safe future value...");
    unsafe { core::ptr::write_volatile(mtimecmp_ptr, safe_future) };

    // 読み戻し確認
    let readback_val = unsafe { core::ptr::read_volatile(mtimecmp_ptr) };
    print!("  MTIMECMP readback: ");
    print_number!(readback_val);
    println!();

    if readback_val == safe_future {
        println!("✓ MTIMECMP access successful");
    } else {
        println!("✗ MTIMECMP write/read mismatch");
    }
}

/// MSIP 段階的アクセステスト
fn test_msip_access_staged() {
    println!("Testing MSIP access (staged approach)...");

    // Stage 1: アドレス範囲チェック
    println!("  Stage 1: Address range validation");
    if MSIP_BASE >= CLINT_BASE && MSIP_BASE < CLINT_BASE + 0x10000 {
        println!("  ✓ MSIP address in valid CLINT range");
    } else {
        println!("  ✗ MSIP address outside CLINT range");
        return;
    }

    // Stage 2: 非常に慎重な読み取り試行
    println!("  Stage 2: Careful read attempt");
    let msip_ptr = MSIP_BASE as *const u32;

    // トラップハンドラが正常動作する状態で試行
    println!("  Attempting MSIP read...");

    // 読み取り試行（例外が発生する可能性あり）
    let msip_val = unsafe {
        // 例外が発生した場合はtrap handlerで処理される
        core::ptr::read_volatile(msip_ptr)
    };

    print!("  MSIP read value: ");
    print_number!(msip_val as u64);
    println!();

    println!("  Stage 3: Write test (if read succeeded)");

    // Stage 3: 書き込みテスト（読み取りが成功した場合のみ）
    let msip_mut_ptr = MSIP_BASE as *mut u32;

    // 0を書き込み（クリア）
    println!("  Writing 0 to MSIP...");
    unsafe { core::ptr::write_volatile(msip_mut_ptr, 0) };

    let msip_after_clear = unsafe { core::ptr::read_volatile(msip_ptr) };
    print!("  MSIP after clear: ");
    print_number!(msip_after_clear as u64);
    println!();

    // 1を書き込み（セット）
    println!("  Writing 1 to MSIP...");
    unsafe { core::ptr::write_volatile(msip_mut_ptr, 1) };

    let msip_after_set = unsafe { core::ptr::read_volatile(msip_ptr) };
    print!("  MSIP after set: ");
    print_number!(msip_after_set as u64);
    println!();

    // 再び0をクリア（安全のため）
    println!("  Clearing MSIP for safety...");
    unsafe { core::ptr::write_volatile(msip_mut_ptr, 0) };

    let msip_final = unsafe { core::ptr::read_volatile(msip_ptr) };
    print!("  MSIP final state: ");
    print_number!(msip_final as u64);
    println!();

    // 結果評価
    if msip_after_set == 1 && msip_final == 0 {
        println!("✓ MSIP access fully functional");
    } else {
        println!("⚠ MSIP access partially working");
    }
}

/// 基本的なMSIP操作テスト
pub fn basic_msip_test() {
    println!("=== BASIC MSIP FUNCTIONALITY TEST ===");

    let msip_ptr = MSIP_BASE as *mut u32;

    println!("Testing basic MSIP operations...");

    // 初期状態確認
    let initial = unsafe { core::ptr::read_volatile(msip_ptr) };
    println_number!("Initial MSIP: ", initial as u64);

    // セット
    unsafe { core::ptr::write_volatile(msip_ptr, 1) };
    let after_set = unsafe { core::ptr::read_volatile(msip_ptr) };
    println_number!("After set: ", after_set as u64);

    // クリア
    unsafe { core::ptr::write_volatile(msip_ptr, 0) };
    let after_clear = unsafe { core::ptr::read_volatile(msip_ptr) };
    println_number!("After clear: ", after_clear as u64);

    if after_set == 1 && after_clear == 0 {
        println!("✓ Basic MSIP operations working");
    } else {
        println!("✗ MSIP operations failed");
    }
}

/// エラー処理付きMSIP読み取り
pub fn safe_msip_read() -> Result<u32, &'static str> {
    let msip_ptr = MSIP_BASE as *const u32;

    // 読み取り試行
    let val = unsafe { core::ptr::read_volatile(msip_ptr) };

    // 基本的な値の妥当性チェック
    if val <= 1 {
        Ok(val)
    } else {
        Err("Invalid MSIP value")
    }
}

/// エラー処理付きMSIP書き込み
pub fn safe_msip_write(value: u32) -> Result<(), &'static str> {
    if value > 1 {
        return Err("Invalid MSIP value (must be 0 or 1)");
    }

    let msip_ptr = MSIP_BASE as *mut u32;
    unsafe { core::ptr::write_volatile(msip_ptr, value) };

    // 書き込み確認
    let readback = unsafe { core::ptr::read_volatile(msip_ptr) };
    if readback == value {
        Ok(())
    } else {
        Err("MSIP write verification failed")
    }
}
