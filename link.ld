ENTRY(_start)

SECTIONS {
    . = 0x80000000;

    .text : {
        *(.text.entry)
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : AT (ADDR(.text) + SIZEOF(.text) + SIZEOF(.rodata)) {
        PROVIDE(__data_start = .);
        *(.data*)
        PROVIDE(__data_end = .);
    }

    PROVIDE(__data_rom_start = LOADADDR(.data));

    .bss : {
        PROVIDE(__bss_start = .);
        *(.bss*)
        *(COMMON)
        PROVIDE(__bss_end = .);
    }
}
