default: tap

tap_if = virttap0

build:
	cargo run --release

tap: build 
	qemu-system-x86_64 -m 64M \
		-nic tap,ifname=virttap0,script=no \
		-serial telnet:localhost:4321,server,nowait \
		-nographic \
		-monitor stdio

user: build
	qemu-system-x86_64 -monitor stdio -nographic -m 64 \
	-netdev user,id=mynet0,tftp=bootloader/build,bootfile=stage0.bin \
	-device e1000,netdev=mynet0 \
	-serial telnet:localhost:4321,server,nowait
