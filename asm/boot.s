.section .text.init
.global _start

_start:
    # Set up stack pointer (highest priority)
    li sp, 0x80100000
    
    # Clear frame pointer
    li fp, 0
    
    # Set up global pointer (very carefully)
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    
    # Initialize basic registers to zero (safety)
    li t0, 0
    li t1, 0
    li t2, 0
    li a0, 0
    li a1, 0
    li a2, 0
    li a3, 0
    
    # Call Rust main function
    call rust_main
    
    # If rust_main returns (should never happen), infinite loop
1:
    nop
    j 1b