unsafe extern "C" {
    unsafe static __bss_start: u8;
    unsafe static __bss_end: u8;
    unsafe static __data_start: u8;
    unsafe static __data_end: u8;
}

pub unsafe fn zero_bss() {
    let start = &__bss_start as *const u8 as *mut u8;
    let end = &__bss_end as *const u8;
    let len = end as usize - start as usize;

    if len > 0 {
        core::ptr::write_bytes(start, 0, len);
    }
}

pub unsafe fn init_data() {
    // データセクションは既にRAMに配置されているので、
    // この関数は何もしません（QEMU virtマシンの場合）
    // 必要に応じて初期化コードをここに追加
}
