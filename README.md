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
* NIC working, we get the NIC from the PCI devices list, we have an E1000 network driver which does basic send recieve. Packet structure parsing
and reading are in but need a lot of work. We can handle ARP and DHCP right now in a very static way.

## How to
1. From the root Directory
1. (Windows) ```nmake``` (Linux) ```make``` 
