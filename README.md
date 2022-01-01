# AzphOS

## Requirements
1. Rust (Currently on: ```1.58.0```)
2. Rust target ```i586-pc-windows-msvc``` nightly
2. ``LLD 13.0.0`` (This is cross platform)
3. ```nasm```
4. ```qemu-system-x86_64```

## Currently implemented
* Serial Driver (Printing Only), the make file adds a telnet connection for ```localhost::4321``` which will be available when the machine boots
* VGA Driver (Printing Text Only)
* Get DateTime from CMOS
* PCI get a list of PCI devices and parse the 128-bits of information
* ACPI started, we got the RSD Pointer and then the RSD Table which lead us to the ACPI tables (WIP)

## How to
1. From the root Directory
1. (Windows) ```nmake``` (Linux) ```make``` 
