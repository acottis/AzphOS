# AzphOS

## Requirements
1. Rust (Currently on: ```1.65.0```)
2. Rust target i586-pc-windows-msvc nightly ```rustup target add i586-pc-windows-msvc```
2. ``LLD 13.0.0`` (This is cross platform) https://github.com/llvm/llvm-project/releases/tag/llvmorg-13.0.0 
3. ```nasm``` https://www.nasm.us/pub/nasm/releasebuilds/?C=M;O=D 
4. ```qemu-system-x86_64``` https://qemu.weilnetz.de/w64/ 

## Currently implemented
* Serial Driver (Printing Only), the make file adds a telnet connection for ```localhost::4321``` which will be available when the machine boots
* VGA Driver (Printing Text Only)
* Get DateTime from CMOS
* PCI get a list of PCI devices and parse the 128-bits of information
* ACPI started, we got the RSD Pointer and then the RSD Table which lead us to the ACPI tables (WIP)
* NIC working, we get the NIC from the PCI devices list, we have an E1000 network driver which does basic send recieve. Packet structure parsing
and reading are in but need a lot of work. We can handle ARP and DHCP right now in a very static way.

## How to
1. From the root Directory
2. (Windows) ```nmake``` (Linux) ```make```; 
3. ```nmake tap``` to boot using a TAP adapter to give VM access to local network
