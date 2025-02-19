cargo xbuild-esp32c3 --release
if %errorlevel% neq 0 exit /b %errorlevel%

c:\Espressif\tools\riscv32-esp-elf\esp-13.2.0_20240530\riscv32-esp-elf\bin\riscv32-esp-elf-readelf -hSl target\riscv32imc-unknown-none-elf\release\justboot

c:\tools\esptool\esptool.exe --chip=esp32c3 elf2image  target\riscv32imc-unknown-none-elf\release\justboot --output justboot.bin --flash_mode dio

c:\Espressif\tools\riscv32-esp-elf\esp-13.2.0_20240530\riscv32-esp-elf\bin\riscv32-esp-elf-size target\riscv32imc-unknown-none-elf\release\justboot

espflash flash --bootloader=justboot.bin --monitor example.elf
