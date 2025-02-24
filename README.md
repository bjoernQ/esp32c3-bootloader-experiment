# ESP32-C3 Bootloader Experiment

Just a bare-minimum bootloader. Things are mostly hardcoded.

## Build

cargo xbuild-esp32XXX --release
esptool --chip=esp32XXX elf2image  target\riscv32imc-unknown-none-elf\release\justboot --output justboot.bin --flash_mode dio

## Status

Just for booting esp-hal baremetal. esp-idf images won't work (for now).

- ESP32-C3
    - works fine so far

- added ESP32-C2
    - works so far for, wifi etc. only on chips with a 40MHz xtal

- added ESP32-C6
    - works fine so far

- added ESP32-H2
    - works fine so far

- ESP32-S3
    - works fine so far

- ESP32-S2
    - something simple works, clocks are off after trying to change CPU clock, clocks (e.g. timers) slower than assumed

- ESP32
    - like ESP32-S2
