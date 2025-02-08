#![no_std]
#![no_main]

use esp_hal::main;
use esp_println::println;

#[panic_handler]
fn panic<'a, 'b>(_pi: &'a core::panic::PanicInfo<'b>) -> ! {
    println!("💀");
    loop {}
}

#[main]
fn main() -> ! {
    esp_hal::init(esp_hal::Config::default());
    println!("I'm just a humble bootloader 🦀\n");

    let spiconfig = unsafe { ets_efuse_get_spiconfig() };

    pub const FLASH_SIZE: u32 = 0x1000000;
    pub const FLASH_STATUS_MASK: u32 = 0xFFFF;
    pub const FLASH_SECTOR_SIZE: u32 = 4096;
    pub const PAGE_SIZE: u32 = 0x4000;
    pub const FLASH_BLOCK_SIZE: u32 = 65536;

    // init flash
    let config_result = unsafe {
        esp_rom_spiflash_config_param(
            0,
            FLASH_SIZE,        // total_size
            FLASH_BLOCK_SIZE,  // block_size
            FLASH_SECTOR_SIZE, // sector_size
            PAGE_SIZE,         // page_size
            FLASH_STATUS_MASK, // status_mask
        )
    };

    if config_result == 0 {
        unsafe { esp_rom_spiflash_attach(spiconfig, false) };
    } else {
        println!("Nope 💀");
    }

    // init mmu
    let autoload = unsafe {
        Cache_MMU_Init();
        Cache_Enable_ICache(1);

        let autoload = Cache_Suspend_ICache();
        Cache_Invalidate_ICache_All();

        autoload
    };

    let image_offset = 0x10_000;

    let mut buffer = [0u8; 4096];

    read_flash(image_offset, 4096, &mut buffer);

    let header = unsafe { &*(buffer.as_ptr() as *const EspImageHeader) };

    let segments = header.segment_count as usize;
    let entry_addr = header.entry_addr as usize;

    println!("Segments: {}", segments);
    println!("entry_addr: {:x}", entry_addr);

    let mut flash_addr = image_offset + core::mem::size_of::<EspImageHeader>() as u32;
    for segment in 0..segments {
        read_flash(flash_addr, 4096, &mut buffer);

        let segment_header = unsafe { &*(buffer.as_ptr() as *const EspImageSegmentHeader) };

        let load_addr = segment_header.load_addr as usize;
        let data_len = segment_header.data_len as usize;

        println!(
            "Segment {}: {:x} - {:x}",
            segment,
            load_addr,
            load_addr + data_len
        );

        if load_addr == 0 {
            println!("skip");
        } else {
            if is_ram(load_addr) {
                println!("Copying to RAM");
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
                let blocks = (data_len as u32).div_ceil(64 * 1024);

                let vaddr = vaddr - 0x20;
                let paddr = paddr - 0x20;

                println!("vaddr: {:x}, paddr: {:x}, blocks: {}", vaddr, paddr, blocks);

                let res = if is_drom(vaddr) {
                    unsafe { Cache_Dbus_MMU_Set(0, vaddr, paddr, 64, blocks, 0) }
                } else {
                    unsafe { Cache_Ibus_MMU_Set(0, vaddr, paddr, 64, blocks, 0) }
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

fn is_drom(addr: u32) -> bool {
    addr >= 0x3C00_0000 && addr <= 0x3C7F_FFFF
}

fn is_ram(addr: usize) -> bool {
    !(addr >= 0x3C00_0000 && addr <= 0x3C7F_FFFF || addr >= 0x4200_0000 && addr <= 0x427F_FFFF)
}

fn read_flash(flash_addr: u32, len: usize, data: &mut [u8]) {
    // read from flash
    if unsafe { esp_rom_spiflash_read(flash_addr, data.as_mut_ptr(), len as u32) } != 0 {
        println!("F");
    }
}

extern "C" {
    // flash functions
    fn esp_rom_spiflash_attach(config: u32, legacy: bool);

    fn esp_rom_spiflash_config_param(
        device_id: u32,
        chip_size: u32,
        block_size: u32,
        sector_size: u32,
        page_size: u32,
        status_mask: u32,
    ) -> u32;

    fn ets_efuse_get_spiconfig() -> u32;

    fn esp_rom_spiflash_read(src_addr: u32, data: *mut u8, len: u32) -> i32;

    // mmu functions
    fn Cache_Suspend_ICache() -> u32;
    fn Cache_Resume_ICache(val: u32);
    fn Cache_Invalidate_ICache_All();
    fn Cache_Ibus_MMU_Set(
        ext_ram: u32,
        vaddr: u32,
        paddr: u32,
        psize: u32,
        num: u32,
        fixed: u32,
    ) -> i32;

    fn Cache_Dbus_MMU_Set(
        ext_ram: u32,
        vaddr: u32,
        paddr: u32,
        psize: u32,
        num: u32,
        fixed: u32,
    ) -> i32;

    fn Cache_MMU_Init();

    fn Cache_Enable_ICache(autoload: u32);
}

/// Main header of binary image
#[repr(C, packed)]
struct EspImageHeader {
    magic: u8,              // Magic word ESP_IMAGE_HEADER_MAGIC
    segment_count: u8,      // Count of memory segments
    spi_mode: u8,           // flash read mode (esp_image_spi_mode_t as u8)
    spi_speed_and_size: u8, // flash frequency (esp_image_spi_freq_t as u8) and  flash chip size (esp_image_flash_size_t as u8)
    entry_addr: u32,        // Entry address
    wp_pin: u8,             // WP pin when SPI pins set via efuse (read by ROM bootloader,
    // the IDF bootloader uses software to configure the WP
    // pin and sets this field to 0xEE=disabled)
    spi_pin_drv: [u8; 3], // Drive settings for the SPI flash pins (read by ROM bootloader)
    chip_id: EspChipId,   // Chip identification number
    min_chip_rev: u8,     // Minimal chip revision supported by image
    // After the Major and Minor revision eFuses were introduced into the chips, this field is no longer used.
    // But for compatibility reasons, we keep this field and the data in it.
    // Use min_chip_rev_full instead.
    // The software interprets this as a Major version for most of the chips and as a Minor version for the ESP32-C3.
    min_chip_rev_full: u16, // Minimal chip revision supported by image, in format: major * 100 + minor
    max_chip_rev_full: u16, // Maximal chip revision supported by image, in format: major * 100 + minor
    reserved: [u8; 4],      // Reserved bytes in additional header space, currently unused
    hash_appended: u8, // If 1, a SHA256 digest "simple hash" (of the entire image) is appended after the checksum.
                       // Included in image length. This digest
                       // is separate to secure boot and only used for detecting corruption.
                       // For secure boot signed images, the signature
                       // is appended after this (and the simple hash is included in the signed data).
}

/// Header of binary image segment
#[repr(C, packed)]
struct EspImageSegmentHeader {
    load_addr: u32, // Address of segment
    data_len: u32,  // Length of data
}

/// ESP chip ID
#[repr(u16)]
#[allow(unused)]
enum EspChipId {
    Esp32 = 0x0000,        // chip ID: ESP32
    Esp32S2 = 0x0002,      // chip ID: ESP32-S2
    Esp32C3 = 0x0005,      // chip ID: ESP32-C3
    Esp32S3 = 0x0009,      // chip ID: ESP32-S3
    Esp32C2 = 0x000C,      // chip ID: ESP32-C2
    Esp32C6 = 0x000D,      // chip ID: ESP32-C6
    Esp32H2 = 0x0010,      // chip ID: ESP32-H2
    Esp32P4 = 0x0012,      // chip ID: ESP32-P4
    Esp32C5Beta3 = 0x0011, // chip ID: ESP32-C5 beta3 (MPW)
    Esp32C5Mp = 0x0017,    // chip ID: ESP32-C5 MP
    Invalid = 0xFFFF, // Invalid chip ID (we defined it to make sure the esp_chip_id_t is 2 bytes size)
}
