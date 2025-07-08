MEMORY
{
  RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

ENTRY(_start)

SECTIONS
{
  . = 0x80000000;

  .text : {
    KEEP(*(.text.init));
    *(.text);
    *(.text.*);
  } > RAM

  .rodata : ALIGN(4) {
    *(.rodata);
    *(.rodata.*);
  } > RAM

  .data : ALIGN(4) {
    __data_start = .;
    *(.data);
    *(.data.*);
    __data_end = .;
  } > RAM

  .bss : ALIGN(4) {
    __bss_start = .;
    *(.bss);
    *(.bss.*);
    *(COMMON);
    __bss_end = .;
  } > RAM

  /* グローバルポインター */
  __global_pointer$ = MIN(__data_start + 0x800, MAX(__data_start + 0x800, __bss_end - 0x800));

  /DISCARD/ : {
    *(.eh_frame*);
    *(.debug*);
    *(.comment);
  }
}