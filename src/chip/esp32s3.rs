use esp_println::println;

pub const MMU_ACCESS_FLASH: u32 = 0;

pub fn init_flash() {
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
}

pub fn init_mmu() -> u32 {
    // init mmu
    unsafe {
        Cache_MMU_Init();
        Cache_Enable_ICache(1);

        let autoload = rom_Cache_Suspend_ICache();
        Cache_Invalidate_ICache_All();

        autoload
    }
}

pub fn resume_mmu(autoload: u32) {
    unsafe {
        Cache_Resume_ICache(autoload);
    }
}

pub fn is_drom(addr: u32) -> bool {
    addr >= 0x3C00_0000 && addr <= 0x3C7F_FFFF
}

pub fn is_ram(addr: usize) -> bool {
    !(addr >= 0x3C00_0000 && addr <= 0x3C7F_FFFF || addr >= 0x4200_0000 && addr <= 0x427F_FFFF)
}

pub fn read_flash(flash_addr: u32, len: usize, data: &mut [u8]) {
    // read from flash
    if unsafe { esp_rom_spiflash_read(flash_addr, data.as_mut_ptr(), len as u32) } != 0 {
        println!("F");
    }
}

pub fn dbus_mmu_set(vaddr: u32, paddr: u32, psize: u32, num: u32, fixed: u32) -> i32 {
    unsafe { Cache_Dbus_MMU_Set(MMU_ACCESS_FLASH, vaddr, paddr, psize, num, fixed) }
}

pub fn ibus_mmu_set(vaddr: u32, paddr: u32, psize: u32, num: u32, fixed: u32) -> i32 {
    unsafe { Cache_Ibus_MMU_Set(MMU_ACCESS_FLASH, vaddr, paddr, psize, num, fixed) }
}

extern "C" {
    // flash functions
    pub fn esp_rom_spiflash_attach(config: u32, legacy: bool);

    pub fn esp_rom_spiflash_config_param(
        device_id: u32,
        chip_size: u32,
        block_size: u32,
        sector_size: u32,
        page_size: u32,
        status_mask: u32,
    ) -> u32;

    pub fn ets_efuse_get_spiconfig() -> u32;

    pub fn esp_rom_spiflash_read(src_addr: u32, data: *mut u8, len: u32) -> i32;

    // mmu functions
    pub fn rom_Cache_Suspend_ICache() -> u32;
    pub fn Cache_Resume_ICache(val: u32);
    pub fn Cache_Invalidate_ICache_All();
    pub fn Cache_Ibus_MMU_Set(
        ext_ram: u32,
        vaddr: u32,
        paddr: u32,
        psize: u32,
        num: u32,
        fixed: u32,
    ) -> i32;

    pub fn Cache_Dbus_MMU_Set(
        ext_ram: u32,
        vaddr: u32,
        paddr: u32,
        psize: u32,
        num: u32,
        fixed: u32,
    ) -> i32;

    pub fn Cache_MMU_Init();

    pub fn Cache_Enable_ICache(autoload: u32);
}
