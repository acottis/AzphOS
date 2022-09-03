[![Rust](https://github.com/acottis/AzphOS/actions/workflows/rust.yml/badge.svg)](https://github.com/acottis/AzphOS/actions/workflows/rust.yml)

# AzphOS
The allocationless OS! For performance.

## Requirements to compile
1. Rust (Currently on: ```1.65.0```)
2. Rust target i586-pc-windows-msvc nightly ```rustup target add i586-pc-windows-msvc```
2. ``LLD 13.0.0`` (This is cross platform) https://github.com/llvm/llvm-project/releases/tag/llvmorg-13.0.0 
3. ```nasm``` https://www.nasm.us/pub/nasm/releasebuilds/?C=M;O=D 
4. ```qemu-system-x86_64``` https://qemu.weilnetz.de/w64/

## Network Requirements to pxe boot (Optional)
1. DHCP Server that can set PXE options https://github.com/acottis/dhcp-server
2. Tftp Server that can send the OS to the BIOS https://github.com/acottis/tftp-server
3. Tap adapter

## Currently implemented
* Serial Driver (Printing Only), the make file adds a telnet connection for ```localhost::4321``` which will be available when the machine boots
* VGA Driver (Printing Text Only)
* Get DateTime from CMOS
* PCI get a list of PCI devices and parse the 128-bits of information
* ACPI started, we got the RSD Pointer and then the RSD Table which lead us to the ACPI tables (WIP)
* NIC working, we get the NIC from the PCI devices list, we have an E1000 network driver which does basic send recieve. Packet structure parsing
and reading are in but need a lot of work. We can handle ARP and DHCP right now in a very static way.

## How to
3. From the project root Directory
2. (Windows) ```nmake``` (Linux) ```make```; 


## How to Network Boot
1. Set up Tftp to host `stage0.bin`
2. Set up DHCP to point to the Tftp
3. From the project root Directory
3. ```nmake tap``` to boot using a TAP adapter to give VM access to local network