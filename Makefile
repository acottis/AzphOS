default: local

build:
	cargo run

tap: build
	qemu-system-x86_64 -monitor stdio -nographic \
	-netdev tap,id=mynet0,ifname=mytap \
	-device e1000,netdev=mynet0,bootindex=0 -m 64 \
	-serial telnet:localhost:4321,server,nowait

local: build
	qemu-system-x86_64 -monitor stdio -nographic \
	-nic user,ipv6=off,model=virtio-net-pci,tftp=bootloader/build,bootfile=stage0.bin -m 64 \
	-serial telnet:localhost:4321,server,nowait