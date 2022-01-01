default: boot

build:
	cargo run

boot: build
	qemu-system-x86_64 -monitor stdio -nographic \
 	-netdev user,id=net0,ipv6=off,tftp=bootloader/build,bootfile=stage0.bin	\
	-device e1000,netdev=net0,bootindex=0 -m 64 \
	-serial telnet:localhost:4321,server,nowait
	
#	-nic user,ipv6=off,model=virtio-net-pci,tftp=bootloader/build,bootfile=stage0.bin,bootindex=0
	