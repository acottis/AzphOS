default: boot

build:
	cargo run
	nasm bootloader/asm/stage0.asm -f bin -o bootloader/build/stage0.bin

boot: build
	qemu-system-x86_64 -monitor stdio \
	-netdev user,id=net0,tftp=bootloader/build,bootfile=stage0.bin	\
	-device e1000,netdev=net0,bootindex=0 -m 32 \
	