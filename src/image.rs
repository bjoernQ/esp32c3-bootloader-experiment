/// Main header of binary image
#[repr(C, packed)]
pub struct EspImageHeader {
    pub magic: u8,              // Magic word ESP_IMAGE_HEADER_MAGIC
    pub segment_count: u8,      // Count of memory segments
    pub spi_mode: u8,           // flash read mode (esp_image_spi_mode_t as u8)
    pub spi_speed_and_size: u8, // flash frequency (esp_image_spi_freq_t as u8) and  flash chip size (esp_image_flash_size_t as u8)
    pub entry_addr: u32,        // Entry address
    pub wp_pin: u8,             // WP pin when SPI pins set via efuse (read by ROM bootloader,
    // the IDF bootloader uses software to configure the WP
    // pin and sets this field to 0xEE=disabled)
    pub spi_pin_drv: [u8; 3], // Drive settings for the SPI flash pins (read by ROM bootloader)
    pub chip_id: EspChipId,   // Chip identification number
    pub min_chip_rev: u8,     // Minimal chip revision supported by image
    // After the Major and Minor revision eFuses were introduced into the chips, this field is no longer used.
    // But for compatibility reasons, we keep this field and the data in it.
    // Use min_chip_rev_full instead.
    // The software interprets this as a Major version for most of the chips and as a Minor version for the ESP32-C3.
    pub min_chip_rev_full: u16, // Minimal chip revision supported by image, in format: major * 100 + minor
    pub max_chip_rev_full: u16, // Maximal chip revision supported by image, in format: major * 100 + minor
    pub reserved: [u8; 4],      // Reserved bytes in additional header space, currently unused
    pub hash_appended: u8, // If 1, a SHA256 digest "simple hash" (of the entire image) is appended after the checksum.
                           // Included in image length. This digest
                           // is separate to secure boot and only used for detecting corruption.
                           // For secure boot signed images, the signature
                           // is appended after this (and the simple hash is included in the signed data).
}

/// Header of binary image segment
#[repr(C, packed)]
pub struct EspImageSegmentHeader {
    pub load_addr: u32, // Address of segment
    pub data_len: u32,  // Length of data
}

/// ESP chip ID
#[repr(u16)]
#[allow(unused)]
pub enum EspChipId {
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
