// RISC-V CSR操作ユーティリティ

// mtvec (Machine Trap Vector Base Address)
pub unsafe fn write_mtvec(addr: usize) {
    core::arch::asm!("csrw mtvec, {}", in(reg) addr);
}

pub fn read_mtvec() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mtvec", out(reg) val);
    }
    val
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

// mie (Machine Interrupt Enable) レジスタ操作
pub fn read_mie() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("csrr {}, mie", out(reg) val);
    }
    val
}

pub unsafe fn write_mie(val: usize) {
    core::arch::asm!("csrw mie, {}", in(reg) val);
}

// mstatus レジスタ書き込み
pub unsafe fn write_mstatus(val: usize) {
    core::arch::asm!("csrw mstatus, {}", in(reg) val);
}

// 割り込み制御のヘルパー関数

/// Machine Timer Interrupt Enable (MTIE) を有効化
pub unsafe fn enable_machine_timer_interrupt() {
    let mut mie = read_mie();
    mie |= 1 << 7; // MTIE bit (bit 7)
    write_mie(mie);
}

/// Machine External Interrupt Enable (MEIE) を有効化
pub unsafe fn enable_machine_external_interrupt() {
    let mut mie = read_mie();
    mie |= 1 << 11; // MEIE bit (bit 11)
    write_mie(mie);
}

/// Machine Software Interrupt Enable (MSIE) を有効化
pub unsafe fn enable_machine_software_interrupt() {
    let mut mie = read_mie();
    mie |= 1 << 3; // MSIE bit (bit 3)
    write_mie(mie);
}

/// グローバル割り込みを有効化（mstatusのMIE bit）
pub unsafe fn enable_global_interrupts() {
    let mut mstatus = read_mstatus();
    mstatus |= 1 << 3; // MIE bit (bit 3)
    write_mstatus(mstatus);
}

/// グローバル割り込みを無効化
pub unsafe fn disable_global_interrupts() {
    let mut mstatus = read_mstatus();
    mstatus &= !(1 << 3); // MIE bit をクリア
    write_mstatus(mstatus);
}

/// 割り込み状態を取得
pub fn interrupts_enabled() -> bool {
    let mstatus = read_mstatus();
    (mstatus & (1 << 3)) != 0
}
