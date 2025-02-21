/*
 * SPDX-FileCopyrightText: 2021-2024 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: Apache-2.0
 */
/* Simplified memory map for the bootloader.
 * Make sure the bootloader can load into main memory without overwriting itself.
 */


MEMORY
{
  iram_seg (RWX) :                  org = 0x4004A000, len = 0x4000  /* SRAM part of block 12 and 13 */
  iram_loader_seg (RWX) :           org = 0x4004E000, len = 0x7000  /* SRAM part of block 13, Block 14 & part of 15 */
  dram_seg (RW) :                   org = 0x3FFE5000, len = 0x5B00  /* Part SRAM Blocks 15 & 16, ROM static buffer starts at end of this region (reclaimed after app runs) */

  /* RTC fast memory (executable). Persists over deep sleep. Only for core 0 (PRO_CPU) */
  rtc_fast_seg(RWX) : ORIGIN = 0x40070000, len = 8k

  /* RTC fast memory (same block as above), viewed from data bus. Only for core 0 (PRO_CPU) */
  rtc_fast_dram_seg(RW)  : ORIGIN = 0x3ff9e000, len = 8k

  /* RTC slow memory (data accessible). Persists over deep sleep. */
  rtc_slow_seg(RW)       : ORIGIN = 0x50000000, len = 8k
}



REGION_ALIAS("iram", iram_loader_seg);
REGION_ALIAS("dram", dram_seg);
REGION_ALIAS("rtc_fast", rtc_fast_seg);
REGION_ALIAS("rtc_slow", rtc_slow_seg);

ENTRY(ESP32Reset)

PROVIDE(__pre_init = DefaultPreInit);
PROVIDE(__zero_bss = default_mem_hook);
PROVIDE(__init_data = default_mem_hook);
PROVIDE(__post_init = default_post_init);

PROVIDE(__level_1_interrupt = handle_interrupts);
PROVIDE(__level_2_interrupt = handle_interrupts);
PROVIDE(__level_3_interrupt = handle_interrupts);

INCLUDE device.x

INCLUDE "hal-defaults.x"
INCLUDE "rom-functions.x"

/* high level exception/interrupt routines, which can be override with Rust functions */
PROVIDE(__exception = __default_exception);
PROVIDE(__user_exception = __default_user_exception);
PROVIDE(__double_exception = __default_double_exception);
PROVIDE(__level_1_interrupt = __default_interrupt);
PROVIDE(__level_2_interrupt = __default_interrupt);
PROVIDE(__level_3_interrupt = __default_interrupt);
PROVIDE(__level_4_interrupt = __default_interrupt);
PROVIDE(__level_5_interrupt = __default_interrupt);
PROVIDE(__level_6_interrupt = __default_interrupt);
PROVIDE(__level_7_interrupt = __default_interrupt);

/* high level CPU interrupts */
PROVIDE(Timer0 = __default_user_exception);
PROVIDE(Timer1 = __default_user_exception);
PROVIDE(Timer2 = __default_user_exception);
PROVIDE(Timer3 = __default_user_exception);
PROVIDE(Profiling = __default_user_exception);
PROVIDE(NMI = __default_user_exception);
PROVIDE(Software0 = __default_user_exception);
PROVIDE(Software1 = __default_user_exception);

/* low level exception/interrupt, which must be overridden using naked functions */
PROVIDE(__naked_user_exception = __default_naked_exception);
PROVIDE(__naked_kernel_exception = __default_naked_exception);
PROVIDE(__naked_double_exception = __default_naked_double_exception);
PROVIDE(__naked_level_2_interrupt = __default_naked_level_2_interrupt);
PROVIDE(__naked_level_3_interrupt = __default_naked_level_3_interrupt);
PROVIDE(__naked_level_4_interrupt = __default_naked_level_4_interrupt);
PROVIDE(__naked_level_5_interrupt = __default_naked_level_5_interrupt);
PROVIDE(__naked_level_6_interrupt = __default_naked_level_6_interrupt);
PROVIDE(__naked_level_7_interrupt = __default_naked_level_7_interrupt);


/* needed to force inclusion of the vectors */
EXTERN(__default_exception);
EXTERN(__default_double_exception);
EXTERN(__default_interrupt);

EXTERN(__default_naked_exception);
EXTERN(__default_naked_double_exception);
EXTERN(__default_naked_level_2_interrupt);
EXTERN(__default_naked_level_3_interrupt);
EXTERN(__default_naked_level_4_interrupt);
EXTERN(__default_naked_level_5_interrupt);
EXTERN(__default_naked_level_6_interrupt);
EXTERN(__default_naked_level_7_interrupt);

SECTIONS {
.vectors :
  {
    . = ALIGN(0x400);
    /* 
      Each vector has 64 bytes that it must fit inside. For each vector we calculate the size of the previous one, 
      and subtract that from 64 and start the new vector there.
    */
    _init_start = ABSOLUTE(.);
    . = ALIGN(64);
    KEEP(*(.WindowOverflow4.text));
    . = ALIGN(64);
    KEEP(*(.WindowUnderflow4.text));
    . = ALIGN(64);
    KEEP(*(.WindowOverflow8.text));
    . = ALIGN(64);
    KEEP(*(.WindowUnderflow8.text));
    . = ALIGN(64);
    KEEP(*(.WindowOverflow12.text));
    . = ALIGN(64);
    KEEP(*(.WindowUnderflow12.text));
    . = ALIGN(64);
    KEEP(*(.Level2InterruptVector.text));
    . = ALIGN(64);
    KEEP(*(.Level3InterruptVector.text));
    . = ALIGN(64);
    KEEP(*(.Level4InterruptVector.text));
    . = ALIGN(64);
    KEEP(*(.Level5InterruptVector.text));
    . = ALIGN(64);
    KEEP(*(.DebugExceptionVector.text));
    . = ALIGN(64);
    KEEP(*(.NMIExceptionVector.text));
    . = ALIGN(64);
    KEEP(*(.KernelExceptionVector.text));
    . = ALIGN(64);
    KEEP(*(.UserExceptionVector.text));
    . = ALIGN(128);
    KEEP(*(.DoubleExceptionVector.text));
    . = ALIGN(64);
    . = ALIGN(0x400);
    _init_end = ABSOLUTE(.);
  } > iram

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

 .rtc_slow.text : {
   . = ALIGN(4);
   *(.rtc_slow.literal .rtc_slow.text .rtc_slow.literal.* .rtc_slow.text.*)
   . = ALIGN(4);
  } > rtc_slow_seg

  .rtc_slow.data :
  {
    . = ALIGN(4);
    _rtc_slow_data_start = ABSOLUTE(.);
    *(.rtc_slow.data .rtc_slow.data.*)
    _rtc_slow_data_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_slow_seg

 .rtc_slow.bss (NOLOAD) :
  {
    . = ALIGN(4);
    _rtc_slow_bss_start = ABSOLUTE(.);
    *(.rtc_slow.bss .rtc_slow.bss.*)
    _rtc_slow_bss_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_slow_seg

 .rtc_slow.persistent (NOLOAD) :
  {
    . = ALIGN(4);
    _rtc_slow_persistent_start = ABSOLUTE(.);
    *(.rtc_slow.persistent .rtc_slow.persistent.*)
    _rtc_slow_persistent_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_slow_seg

  .rtc_fast.text : {
   . = ALIGN(4);
   *(.rtc_fast.literal .rtc_fast.text .rtc_fast.literal.* .rtc_fast.text.*)
   . = ALIGN(4);
  } > rtc_fast_seg
  
  .rtc_fast.data :
  {
    . = ALIGN(4);
    _rtc_fast_data_start = ABSOLUTE(.);
    *(.rtc_fast.data .rtc_fast.data.*)
    _rtc_fast_data_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_fast_seg

 .rtc_fast.bss (NOLOAD) :
  {
    . = ALIGN(4);
    _rtc_fast_bss_start = ABSOLUTE(.);
    *(.rtc_fast.bss .rtc_fast.bss.*)
    _rtc_fast_bss_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_fast_seg

 .rtc_fast.persistent (NOLOAD) :
  {
    . = ALIGN(4);
    _rtc_fast_persistent_start = ABSOLUTE(.);
    *(.rtc_fast.persistent .rtc_fast.persistent.*)
    _rtc_fast_persistent_end = ABSOLUTE(.);
    . = ALIGN(4);
  } > rtc_fast_seg

  .unused (NOLOAD) :
  {
    *(.eh_frame)
  } > rtc_fast
}


PROVIDE(_stack_start = ORIGIN(dram) + LENGTH(iram));
PROVIDE(_stack_start_cpu0 = ORIGIN(dram) + LENGTH(iram));
PROVIDE(__stack_chk_guard = _stack_end + 4096);
