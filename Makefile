default: tap

tap_if = virttap0

build:
	cargo run --release

tap: build
	qemu-system-x86_64 -monitor stdio -nographic \
	-netdev tap,id=mynet0,ifname=$(tap_if),script=no \
	-device e1000,netdev=mynet0,bootindex=0 -m 64 \
	-serial telnet:localhost:4321,server,nowait

user: build
	qemu-system-x86_64 -monitor stdio -nographic -m 64 \
	-netdev user,id=mynet0,tftp=bootloader/build,bootfile=stage0.bin \
	-device e1000,netdev=mynet0 \
	-serial telnet:localhost:4321,server,nowait
