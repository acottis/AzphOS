default: boot

build:
	cargo run

boot: build
	qemu-system-x86_64 -monitor stdio -nographic \
	-netdev tap,id=mynet0,ifname=mytap \
	-device e1000,netdev=mynet0,bootindex=0 -m 64 \
	-serial telnet:localhost:4321,server,nowait
	
# 	-netdev user,id=net0,ipv6=off,tftp=bootloader/build,bootfile=stage0.bin	\
#	-nic user,ipv6=off,model=virtio-net-pci,tftp=bootloader/build,bootfile=stage0.bin,bootindex=0
	