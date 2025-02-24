use esp_println::println;

pub const MMU_ACCESS_FLASH: u32 = 1 << 15;

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
        mmu_init(0);
        mmu_init(1);

        // Cache_MMU_Init();
        // Cache_Enable_ICache(1);

        // let autoload = Cache_Suspend_ICache();
        // Cache_Invalidate_ICache_All();

        // autoload
        1
    }
}

pub fn resume_mmu(autoload: u32) {
    unsafe {
        const DR_REG_DPORT_BASE: usize = 0x3ff00000;
        const DPORT_PRO_CACHE_CTRL1_REG: usize = (DR_REG_DPORT_BASE + 0x044);
        // TODO DPORT_APP_CACHE_CTRL1_REG

        (DPORT_PRO_CACHE_CTRL1_REG as *mut u32)
            .write_volatile((DPORT_PRO_CACHE_CTRL1_REG as *mut u32).read_volatile() & !0b1001);

        Cache_Read_Enable_rom(0);
    }
}

pub fn is_drom(addr: u32) -> bool {
    addr >= 0x3F40_0000 && addr <= 0x3FBF_FFFF
}

pub fn is_ram(addr: usize) -> bool {
    !(addr >= 0x3F40_0000 && addr <= 0x3FBF_FFFF || addr >= 0x400C_2000 && addr <= 0x40BF_FFFF)
}

pub fn read_flash(flash_addr: u32, len: usize, data: &mut [u8]) {
    // read from flash
    if unsafe { esp_rom_spiflash_read(flash_addr, data.as_mut_ptr(), len as u32) } != 0 {
        println!("F");
    }
}

pub fn dbus_mmu_set(vaddr: u32, paddr: u32, psize: u32, num: u32, fixed: u32) -> i32 {
    unsafe {
        cache_flash_mmu_set_rom(0, 0, vaddr, paddr, psize, num)
            | cache_flash_mmu_set_rom(1, 0, vaddr, paddr, psize, num)
    }
}

pub fn ibus_mmu_set(vaddr: u32, paddr: u32, psize: u32, num: u32, fixed: u32) -> i32 {
    unsafe {
        cache_flash_mmu_set_rom(0, 0, vaddr, paddr, psize, num)
            | cache_flash_mmu_set_rom(1, 0, vaddr, paddr, psize, num)
    }
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

    /**
     * @brief Set Flash-Cache mmu mapping.
     *        Please do not call this function in your SDK application.
     *
     * @param  int cpu_no : CPU number, 0 for PRO cpu, 1 for APP cpu.
     *
     * @param  int pod : process identifier. Range 0~7.
     *
     * @param  unsigned int vaddr : virtual address in CPU address space.
     *                              Can be IRam0, IRam1, IRom0 and DRom0 memory address.
     *                              Should be aligned by psize.
     *
     * @param  unsigned int paddr : physical address in Flash.
     *                              Should be aligned by psize.
     *
     * @param  int psize : page size of flash, in kilobytes. Should be 64 here.
     *
     * @param  int num : pages to be set.
     *
     * @return unsigned int: error status
     *                   0 : mmu set success
     *                   1 : vaddr or paddr is not aligned
     *                   2 : pid error
     *                   3 : psize error
     *                   4 : mmu table to be written is out of range
     *                   5 : vaddr is out of range
     */
    pub fn cache_flash_mmu_set_rom(
        cpu_no: u32,
        pid: u32,
        vaddr: u32,
        paddr: u32,
        psize: u32,
        num: u32,
    ) -> i32;

    pub fn mmu_init(cpu: u32);

    /**
     * @brief Enable Cache access for the cpu.
     *        Please do not call this function in your SDK application.
     *
     * @param  int cpu_no : 0 for PRO cpu, 1 for APP cpu.
     *
     * @return None
     */
    pub fn Cache_Read_Enable_rom(cpu_no: u32);
}
