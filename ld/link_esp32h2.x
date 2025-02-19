/*
 * SPDX-FileCopyrightText: 2022-2024 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: Apache-2.0
 */
/** Simplified memory map for the bootloader.
 *  Make sure the bootloader can load into main memory without overwriting itself.
 *
 *  ESP32-H2 ROM static data usage is as follows:
 *  - 0x4083ba78 - 0x4084d380: Shared buffers, used in UART/USB/SPI download mode only
 *  - 0x4084d380 - 0x4084f380: PRO CPU stack, can be reclaimed as heap after RTOS startup
 *  - 0x4084f380 - 0x4084fee0: ROM .bss and .data used in startup code or nonos/early boot (can be freed when IDF runs)
 *  - 0x4084fee0 - 0x40850000: ROM .bss and .data used in startup code and when IDF runs (cannot be freed)
 *
 *  The 2nd stage bootloader can take space up to the end of ROM shared
 *  buffers area (0x4084d380).
 */

/* We consider 0x3fcdc710 to be the last usable address for 2nd stage bootloader stack overhead, dram_seg,
 * and work out iram_seg and iram_loader_seg addresses from there, backwards.
 */

/* These lengths can be adjusted, if necessary: */
bootloader_usable_dram_end = 0x4084cfd0;
bootloader_stack_overhead = 0x2000; /* For safety margin between bootloader data section and startup stacks */
bootloader_dram_seg_len = 0x5000;
bootloader_iram_loader_seg_len = 0x7000;
bootloader_iram_seg_len = 0x2D00;

/* Start of the lower region is determined by region size and the end of the higher region */
bootloader_dram_seg_end = bootloader_usable_dram_end - bootloader_stack_overhead;
bootloader_dram_seg_start = bootloader_dram_seg_end - bootloader_dram_seg_len;
bootloader_iram_loader_seg_start = bootloader_dram_seg_start - bootloader_iram_loader_seg_len;
bootloader_iram_seg_start = bootloader_iram_loader_seg_start - bootloader_iram_seg_len;

MEMORY
{
  iram_seg (RWX) :                  org = bootloader_iram_seg_start, len = bootloader_iram_seg_len
  iram_loader_seg (RWX) :           org = bootloader_iram_loader_seg_start, len = bootloader_iram_loader_seg_len
  dram_seg (RW) :                   org = bootloader_dram_seg_start, len = bootloader_dram_seg_len

  RTC_FAST : ORIGIN = 0x50000000, LENGTH = 0x2000
}


REGION_ALIAS("iram", iram_loader_seg);
REGION_ALIAS("dram", dram_seg);
REGION_ALIAS("rtc_fast", RTC_FAST);

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

 .rtc_fast.bss (NOLOAD) :
  {
    . = ALIGN(4);
    _rtc_fast_bss_start = ABSOLUTE(.);
    *(.rtc_fast.bss .rtc_fast.bss.*)
    _rtc_fast_bss_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_fast

 .rtc_fast.persistent (NOLOAD) :
  {
    . = ALIGN(4);
    _rtc_fast_persistent_start = ABSOLUTE(.);
    *(.rtc_fast.persistent .rtc_fast.persistent.*)
    _rtc_fast_persistent_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_fast

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
