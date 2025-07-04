unsafe extern "C" {
    static __bss_start: u32;
    static __bss_end: u32;
    static __data_start: u32;
    static __data_end: u32;
    static __data_rom_start: u32;
}

pub unsafe fn zero_bss() {
    unsafe {
        let mut bss = &__bss_start as *const u32 as *mut u32;
        let end = &__bss_end as *const u32;
        while bss < end as *mut u32 {
            core::ptr::write_volatile(bss, 0);
            bss = bss.add(1);
        }
    }
}

pub unsafe fn init_data() {
    unsafe {
        let mut src = &__data_rom_start as *const u32;
        let mut dst = &__data_start as *const u32 as *mut u32;
        let end = &__data_end as *const u32 as *mut u32;
        while dst < end {
            core::ptr::write_volatile(dst, core::ptr::read_volatile(src));
            dst = dst.add(1);
            src = src.add(1);
        }
    }
}
