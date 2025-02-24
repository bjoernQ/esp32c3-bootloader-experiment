use esp_println::println;

pub const MMU_ACCESS_FLASH: u32 = 1 << 15;

pub fn init_flash() {
    // super-wdt-autofeed ... doesn't belong here - have a dedicated init function
    const DR_REG_RTCCNTL_BASE: usize = 0x3f408000;
    const RTC_CNTL_SWD_CONF_REG: usize = (DR_REG_RTCCNTL_BASE + 0x00B0);
    const RTC_CNTL_SWD_AUTO_FEED_EN: u32 = 1 << 31;

    unsafe {
        let ptr = RTC_CNTL_SWD_CONF_REG as *mut u32;
        ptr.write_volatile(ptr.read_volatile() | RTC_CNTL_SWD_AUTO_FEED_EN);
    }

    // actually init flash
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
        // Cache_Enable_ICache(1);

        // let autoload = Cache_Suspend_ICache();
        // Cache_Invalidate_ICache_All();

        Cache_Disable_ICache();
        Cache_Disable_DCache();

        1
    }
}

pub fn resume_mmu(autoload: u32) {
    unsafe {
        // enable bus
        const DR_REG_EXTMEM_BASE: u32 = 0x61800000;
        const EXTMEM_PRO_DCACHE_CTRL1_REG: u32 = DR_REG_EXTMEM_BASE + 0x004;
        const EXTMEM_PRO_ICACHE_CTRL1_REG: u32 = DR_REG_EXTMEM_BASE + 0x044;

        (EXTMEM_PRO_DCACHE_CTRL1_REG as *mut u32).write_volatile(
            0b010, /*((EXTMEM_PRO_DCACHE_CTRL1_REG as *mut u32).read_volatile() & !0b101)*/
        );
        (EXTMEM_PRO_ICACHE_CTRL1_REG as *mut u32).write_volatile(
            0b010, /*((EXTMEM_PRO_ICACHE_CTRL1_REG as *mut u32).read_volatile() & !0b101)*/
        );
        // also need for APP cpu

        Cache_Resume_ICache(1);
        Cache_Resume_DCache(1);

        Cache_Invalidate_ICache_All();
        Cache_Invalidate_DCache_All();

        Cache_Enable_DCache(1);
        Cache_Enable_ICache(1);
    }
}

pub fn is_drom(addr: u32) -> bool {
    addr >= 0x3F00_0000 && addr <= 0x3FF7_FFFF
}

pub fn is_ram(addr: usize) -> bool {
    !(addr >= 0x3F00_0000 && addr <= 0x3FF7_FFFF || addr >= 0x4008_0000 && addr <= 0x407F_FFFF)
}

pub fn read_flash(flash_addr: u32, len: usize, data: &mut [u8]) {
    // read from flash
    if unsafe { esp_rom_spiflash_read(flash_addr, data.as_mut_ptr(), len as u32) } != 0 {
        println!("F");
    }
}

pub fn dbus_mmu_set(vaddr: u32, paddr: u32, psize: u32, num: u32, fixed: u32) -> i32 {
    // dbus doesn't seem to work for the used addr ranges?????
    unsafe { Cache_Ibus_MMU_Set(MMU_ACCESS_FLASH, vaddr, paddr, psize, num, fixed) }
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
    pub fn Cache_Suspend_ICache() -> u32;
    pub fn Cache_Suspend_DCache() -> u32;

    pub fn Cache_Resume_ICache(val: u32);
    pub fn Cache_Resume_DCache(val: u32);

    pub fn Cache_Invalidate_ICache_All();
    pub fn Cache_Invalidate_DCache_All();

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

    pub fn Cache_Enable_DCache(autoload: u32);

    pub fn Cache_Disable_ICache();

    pub fn Cache_Disable_DCache();
}
