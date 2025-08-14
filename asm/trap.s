.section .text
.global trap_handler
.align 4

trap_handler:
    # Simple debug output - write 'T' to UART to show we entered trap
    li t0, 0x10000000
    li t1, 84  # ASCII 'T'
    sb t1, 0(t0)
    
    # save minimal registers
    addi sp, sp, -32
    sd ra, 0(sp)
    sd t0, 8(sp)
    sd t1, 16(sp)
    sd a0, 24(sp)
    
    # call the trap handler in Rust
    call rust_trap_handler
    
    # load registers
    ld ra, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld a0, 24(sp)
    addi sp, sp, 32
    
    # return from the trap
    mret
    