// RISC-V CSR操作ユーティリティ

// mtvec (Machine Trap Vector Base Address)
pub unsafe fn write_mtvec(addr: usize) {
    core::arch::asm!("csrw mtvec, {}", in(reg) addr);
}

// mcause (Machine Cause)
pub fn read_mcause() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mcause", out(reg) val);
    }
    val
}

// mepc (Machine Exception Program Counter)
pub fn read_mepc() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mepc", out(reg) val);
    }
    val
}

pub unsafe fn write_mepc(addr: usize) {
    core::arch::asm!("csrw mepc, {}", in(reg) addr);
}

// mstatus
pub fn read_mstatus() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mstatus", out(reg) val);
    }
    val
}
