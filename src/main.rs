#![no_std]
#![no_main]

#[macro_use]
mod console;

mod arch;
mod timer; // タイマモジュールを追加
mod trap;

mod test;

pub const UART0: *mut u8 = 0x1000_0000 as *mut u8;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("Substrix OS booting...");

    // Tests without trap handler
    test::run_all_tests();

    // Initialize trap handler
    trap::init_trap();

    // Initialize timer with correct addresses
    println!("Initializing timer with correct addresses...");

    // 全てのタイマ関数を無効化してテスト
    timer::show_memory_info(); // 一時無効化

    timer::init_timer(); // 一時無効化
    println!("Testing timer delay functionality (fixed version)...");
    timer::safe_delay_test();
    println!("Timer tests completed");

    // Test with trap
    test::run_detailed_tests();

    // タイマ割り込みの安全テスト（割り込み有効化はスキップ）- 一時無効化
    println!("Running safe timer interrupt test...");
    timer::test_timer_interrupts_safe();

    // 危険な割り込み有効化は一時的にコメントアウト
    println!("Enabling timer interrupts...");
    timer::enable_timer_interrupts();

    println!("Boot complete! System ready (all timer functions disabled for debugging).");

    // メインループ - タイマ割り込みを待つ
    let mut counter = 0;
    loop {
        counter += 1;
        if counter % 10000000 == 0 {
            println!("System running... waiting for timer interrupts");
        }
        unsafe {
            core::arch::asm!("wfi"); // Wait for interrupt
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC occurred!");
    if let Some(location) = info.location() {
        print!("Location: ");
        print!(location.file());
        print!(":");
        print_number!(location.line());
        println!();
    }
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
