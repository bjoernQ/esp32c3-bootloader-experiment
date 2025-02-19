#![no_std]
#![no_main]

mod chip;
mod image;

use esp_hal::main;
use esp_println::println;
pub(crate) use chip::*;
pub(crate) use image::*;

#[panic_handler]
fn panic<'a, 'b>(_pi: &'a core::panic::PanicInfo<'b>) -> ! {
    println!("💀");
    loop {}
}

#[main]
fn main() -> ! {
    esp_hal::init(esp_hal::Config::default());
    println!("I'm just a humble bootloader 🦀\n");   

    init_flash();

    // init mmu
    let autoload = unsafe {
        Cache_MMU_Init();
        Cache_Enable_ICache(1);

        #[cfg(not(feature = "esp32s3"))]
        let autoload = Cache_Suspend_ICache();
        #[cfg(feature = "esp32s3")]
        let autoload = rom_Cache_Suspend_ICache();

        Cache_Invalidate_ICache_All();

        autoload
    };

    let image_offset = 0x10_000;

    let mut buffer = [0u8; 128];

    read_flash(image_offset, 128, &mut buffer);

    let header = unsafe { &*(buffer.as_ptr() as *const EspImageHeader) };

    let segments = header.segment_count as usize;
    let entry_addr = header.entry_addr as usize;

    println!("Segments: {}", segments);
    println!("entry_addr: {:x}", entry_addr);

    let mut flash_addr = image_offset + core::mem::size_of::<EspImageHeader>() as u32;
    for segment in 0..segments {
        read_flash(flash_addr, 128, &mut buffer);

        let segment_header = unsafe { &*(buffer.as_ptr() as *const EspImageSegmentHeader) };

        let load_addr = segment_header.load_addr as usize;
        let data_len = segment_header.data_len as usize;

        println!(
            "Segment {}: {:x} - {:x} (len = {:x})",
            segment,
            load_addr,
            load_addr + data_len,
            data_len,
        );

        if load_addr == 0 {
            println!("skip");
        } else {
            if is_ram(load_addr) {
                println!("Copying to RAM from {:x}", flash_addr + core::mem::size_of::<EspImageSegmentHeader>() as u32);
                let dst = load_addr as *mut u8;
                read_flash(
                    flash_addr + core::mem::size_of::<EspImageSegmentHeader>() as u32,
                    data_len,
                    unsafe { core::slice::from_raw_parts_mut(dst, data_len) },
                );
            } else {
                println!("Mapping flash");
                let paddr = flash_addr + core::mem::size_of::<EspImageSegmentHeader>() as u32;
                let vaddr = load_addr as u32;
                let blocks = (data_len as u32 + vaddr % 0x10_000).div_ceil(64 * 1024);

                let vaddr = vaddr - vaddr % 0x10_000;
                let paddr = paddr - paddr % 0x10_000;

                println!("vaddr: {:x}, paddr: {:x}, blocks: {}", vaddr, paddr, blocks);

                let res = if is_drom(vaddr) {
                    dbus_mmu_set(0, vaddr, paddr, 64, blocks, 0)
                } else {
                    ibus_mmu_set(0, vaddr, paddr, 64, blocks, 0)
                };

                if res != 0 {
                    println!("💀 mmuset {}", res);
                }
            }
        }
        flash_addr += core::mem::size_of::<EspImageSegmentHeader>() as u32 + data_len as u32;
    }

    unsafe {
        Cache_Resume_ICache(autoload);
    }

    // jump to entry
    let entry: extern "C" fn() -> ! = unsafe { core::mem::transmute(entry_addr) };

    println!("\nJump! 🏃‍♂️‍➡️\n");
    entry();
}
