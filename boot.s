.section .text.init
.global _start

_start:
    # Set up stack pointer (highest priority)
    li sp, 0x80100000
    
    # Clear frame pointer
    li fp, 0
    
    # Set up global pointer
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    
    # Call Rust main function
    call rust_main
    
    # Infinite loop (should never reach here)
1:
    wfi  # Wait for interrupt (power saving)
    j 1b
