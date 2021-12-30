# AzphOS

## Requirements
1. Rust
2. Rust target ```i586-pc-windows-msvc``` nightly
2. ``LLD 13.0.0`` (This is cross platform)
3. ```nasm```
4. ```qemu-system-x86_64```

## Currently implemented
* Serial Driver (Printing Only), the make file adds a telnet connection for ```localhost::4321``` which will be available when the machine boots

## How to
1. From the root Directory
1. (Windows) ```nmake``` (Linux) ```make``` 
