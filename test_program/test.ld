OUTPUT_ARCH(riscv)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x00000000, LENGTH = 64
}

SECTIONS
{
    .text :
    {
        . = 0;
        KEEP(*(.vector_table))
        KEEP(*(.input))
        KEEP(*(.output))
        . = 16;
        *(.text)
        *(.text*)
        *(.rodata)
        *(.rodata*)
    } > RAM
}

ENTRY(_start)
