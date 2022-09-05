default: tap

build:
	cargo run

tap: build
	qemu-system-x86_64 -monitor stdio -nographic \
	-netdev tap,id=mynet0,ifname=mytap \
	-device e1000,netdev=mynet0,bootindex=0 -m 64 \
	-serial telnet:localhost:4321,server,nowait

user: build
	qemu-system-x86_64 -monitor stdio -nographic -m 64 \
	-netdev user,id=mynet0,tftp=bootloader/build,bootfile=stage0.bin \
	-device e1000,netdev=mynet0 \
	-serial telnet:localhost:4321,server,nowait