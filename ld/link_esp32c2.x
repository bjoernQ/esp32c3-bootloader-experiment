/*
 * SPDX-FileCopyrightText: 2021-2024 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: Apache-2.0
 */
/** Simplified memory map for the bootloader.
 *  Make sure the bootloader can load into main memory without overwriting itself.
 *
 *  ESP32-C2 ROM static data usage is as follows:
 *  - 0x3fccb264 - 0x3fcdcb70: Shared buffers, used in UART/USB/SPI download mode only
 *  - 0x3fcdcb70 - 0x3fcdeb70: PRO CPU stack, can be reclaimed as heap after RTOS startup
 *  - 0x3fcdeb70 - 0x3fce0000: ROM .bss and .data (not easily reclaimable)
 *
 *  The 2nd stage bootloader can take space up to the end of ROM shared
 *  buffers area (0x3fcdcb70).
 */

/* The offset between Dbus and Ibus. Used to convert between 0x403xxxxx and 0x3fcxxxxx addresses. */
iram_dram_offset = 0x6e0000;

/* We consider 0x3fcdcb70 to be the last usable address for 2nd stage bootloader stack overhead, dram_seg,
 * and work out iram_seg and iram_loader_seg addresses from there, backwards.
 */

/* These lengths can be adjusted, if necessary: */
bootloader_usable_dram_end = 0x3fcdcb70;
bootloader_stack_overhead = 0x2000; /* For safety margin between bootloader data section and startup stacks */
bootloader_dram_seg_len = 0x5000;
bootloader_iram_loader_seg_len = 0x7000;
bootloader_iram_seg_len = 0x2800;

/* Start of the lower region is determined by region size and the end of the higher region */
bootloader_dram_seg_end = bootloader_usable_dram_end - bootloader_stack_overhead;
bootloader_dram_seg_start = bootloader_dram_seg_end - bootloader_dram_seg_len;
bootloader_iram_loader_seg_start = bootloader_dram_seg_start - bootloader_iram_loader_seg_len + iram_dram_offset;
bootloader_iram_seg_start = bootloader_iram_loader_seg_start - bootloader_iram_seg_len;

MEMORY
{
  iram_seg (RWX) :                  org = bootloader_iram_seg_start, len = bootloader_iram_seg_len
  iram_loader_seg (RWX) :           org = bootloader_iram_loader_seg_start, len = bootloader_iram_loader_seg_len
  dram_seg (RW) :                   org = bootloader_dram_seg_start, len = bootloader_dram_seg_len
}

REGION_ALIAS("iram", iram_loader_seg);
REGION_ALIAS("dram", dram_seg);

INCLUDE "hal-defaults.x"
INCLUDE "rom-functions.x"


SECTIONS {
  .text : ALIGN(4)
  {
    KEEP(*(.trap));
    *(.trap.*);

    KEEP(*(.init));
    KEEP(*(.init.rust));
    KEEP(*(.text.abort));
    *(.literal .text .literal.* .text.*)
    *(.rwtext.literal .rwtext .rwtext.literal.* .rwtext.*)
  } > iram

  .data : ALIGN(4)
  {
    _data_start = ABSOLUTE(.);
    . = ALIGN (4);

    *(.sdata .sdata.* .sdata2 .sdata2.*);
    *(.data .data.*);
    *(.data1)

    *(.rodata .rodata.*)
    *(.srodata .srodata.*)

    . = ALIGN(4);
  } > dram

  .bss (NOLOAD) : ALIGN(4)
  {
    _bss_start = ABSOLUTE(.);
    . = ALIGN (4);
    *(.dynsbss)
    *(.sbss)
    *(.sbss.*)
    *(.gnu.linkonce.sb.*)
    *(.scommon)
    *(.sbss2)
    *(.sbss2.*)
    *(.gnu.linkonce.sb2.*)
    *(.dynbss)
    *(.sbss .sbss.* .bss .bss.*);
    *(.share.mem)
    *(.gnu.linkonce.b.*)
    *(COMMON)
    _bss_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > dram

  /* must be last segment using RWDATA */
  .stack (NOLOAD) : ALIGN(4)
  {
    . = ALIGN (4);
    _stack_end = ABSOLUTE(.);
    _stack_end_cpu0 = ABSOLUTE(.);
  } > dram

  .unused (NOLOAD) :
  {
    *(.eh_frame)
  } > rtc_fast
}


PROVIDE(_stack_start = ORIGIN(iram) + LENGTH(iram));
PROVIDE(_stack_start_cpu0 = ORIGIN(iram) + LENGTH(iram));
PROVIDE(__stack_chk_guard = _stack_end + 4096);

PROVIDE(UserSoft = DefaultHandler);
PROVIDE(SupervisorSoft = DefaultHandler);
PROVIDE(MachineSoft = DefaultHandler);
PROVIDE(UserTimer = DefaultHandler);
PROVIDE(SupervisorTimer = DefaultHandler);
PROVIDE(MachineTimer = DefaultHandler);
PROVIDE(UserExternal = DefaultHandler);
PROVIDE(SupervisorExternal = DefaultHandler);
PROVIDE(MachineExternal = DefaultHandler);

PROVIDE(ExceptionHandler = DefaultExceptionHandler);

PROVIDE(__post_init = default_post_init);

PROVIDE(_max_hart_id = 0);
