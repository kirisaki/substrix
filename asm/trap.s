.section .text
.global trap_handler
.align 4

trap_handler:
    # より安全なレジスタ保存
    # スタックポインタを確認してから操作
    
    # まず、スタックが有効かチェック（基本的な範囲確認）
    li t0, 0x80000000       # RAM start
    bgtu t0, sp, bad_stack  # sp < RAM start なら危険
    li t0, 0x88000000       # RAM end (128MB)
    bltu t0, sp, bad_stack  # sp > RAM end なら危険
    
    # スタック操作（より慎重に）
    addi sp, sp, -256       # 十分なスペースを確保
    
    # レジスタ保存（より多くのレジスタを安全に保存）
    sd ra,   0(sp)
    sd t0,   8(sp)
    sd t1,  16(sp)
    sd t2,  24(sp)
    sd a0,  32(sp)
    sd a1,  40(sp)
    sd a2,  48(sp)
    sd a3,  56(sp)
    sd a4,  64(sp)
    sd a5,  72(sp)
    sd a6,  80(sp)
    sd a7,  88(sp)
    sd s0,  96(sp)
    sd s1, 104(sp)
    
    # Rustトラップハンドラを呼び出し
    call rust_trap_handler
    
    # レジスタ復帰
    ld ra,   0(sp)
    ld t0,   8(sp)
    ld t1,  16(sp)
    ld t2,  24(sp)
    ld a0,  32(sp)
    ld a1,  40(sp)
    ld a2,  48(sp)
    ld a3,  56(sp)
    ld a4,  64(sp)
    ld a5,  72(sp)
    ld a6,  80(sp)
    ld a7,  88(sp)
    ld s0,  96(sp)
    ld s1, 104(sp)
    
    # スタックポインタ復帰
    addi sp, sp, 256
    
    # トラップから復帰
    mret

bad_stack:
    # スタックが無効な場合の緊急処理
    # UARTに直接エラー出力
    li t0, 0x10000000       # UART address
    li t1, 83               # 'S' for Stack error
    sb t1, 0(t0)
    li t1, 10               # '\n'
    sb t1, 0(t0)
    
    # 無限ループ（復帰不可能）
1:
    nop
    j 1b