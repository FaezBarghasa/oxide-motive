/*
    Linker script for the WeAct STM32H750VBT6 Application
    - Total Flash: 128KB. First 32KB reserved for bootloader.
    - Application Flash: 96KB starting at 0x08008000.
    - 1MB Internal SRAM
      - 128KB DTCM RAM (0x20000000)
      - 512KB AXI SRAM (0x24000000)
      - 64KB SRAM4 (0x38000000)
*/
MEMORY
{
  /* Application firmware region, after 32KB bootloader */
  FLASH (rx)    : ORIGIN = 0x08008000, LENGTH = 96K
  DTCM (rwx)    : ORIGIN = 0x20000000, LENGTH = 128K
  AXI_SRAM (rwx): ORIGIN = 0x24000000, LENGTH = 512K
  SRAM4 (rwx)   : ORIGIN = 0x38000000, LENGTH = 64K
}

/* This is where the call stack will be allocated. */
_stack_start = ORIGIN(DTCM) + LENGTH(DTCM);

ENTRY(Reset);

SECTIONS
{
    .text :
    {
        . = ALIGN(4);
        KEEP(*(.isr_vector))
        *(.text .text.* .rodata .rodata.*)
    } > FLASH

    .itcm :
    {
        . = ALIGN(4);
        _sitcm = .;
        *(.itcm .itcm.*)
        . = ALIGN(4);
        _eitcm = .;
    } > DTCM AT> FLASH
    _litcm = LOADADDR(.itcm);

    .ARM.extab : { *(.ARM.extab* .gnu.linkonce.armextab.*) } > FLASH
    .ARM.exidx : { *(.ARM.exidx* .gnu.linkonce.armexidx.*) } > FLASH
    _etext = LOADADDR(.text) + SIZEOF(.text);

    .data :
    {
        _sdata = .;
        *(.data .data.*)
        _edata = .;
    } > DTCM AT> FLASH
    _sidata = _etext;

    .bss :
    {
        _sbss = .;
        *(.bss .bss.* COMMON)
        _ebss = .;
    } > DTCM

    .axi_sram (NOLOAD) :
    {
        . = ALIGN(4);
        _saxi_sram = .;
        *(.axi_sram .axi_sram.*)
        . = ALIGN(4);
        _eaxi_sram = .;
    } > AXI_SRAM

    .sram4 (NOLOAD) :
    {
        . = ALIGN(4);
        _ssram4 = .;
        *(.sram4 .sram4.*)
        . = ALIGN(4);
        _esram4 = .;
    } > SRAM4

    /DISCARD/ :
    {
        libc.a(*)
        libm.a(*)
        libgcc.a(*)
    }
}