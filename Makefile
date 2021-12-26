default: boot

build:
	cargo run

boot: build
	qemu-system-x86_64 -monitor stdio \
	-netdev user,id=net0,tftp=bootloader/build,bootfile=stage0.bin	\
	-device e1000,netdev=net0,bootindex=0 -m 16 \
	