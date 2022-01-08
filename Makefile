TARGET      := riscv64imac-unknown-none-elf
MODE        := debug
KERNEL_FILE := target/$(TARGET)/$(MODE)/wcore
BIN_FILE    := target/$(TARGET)/$(MODE)/kernel.bin

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

ifndef NCPU
NCPU := 4
endif

QEMU_ARGS = -machine virt \
            -nographic \
            -bios default \
            -device loader,file=$(BIN_FILE),addr=0x80200000 \
			-smp $(NCPU)

.PHONY: doc kernel build clean qemu run dtc debug

build: $(BIN_FILE) 

doc:
	@cargo doc --document-private-items

kernel:
	@cargo build

$(BIN_FILE): kernel
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $@

asm:
	@$(OBJDUMP) -d $(KERNEL_FILE) | less

clean:
	@cargo clean

# run qemu
qemu: build
	@qemu-system-riscv64 $(QEMU_ARGS)

# run qemu in debug mode
debug: build
	@qemu-system-riscv64 -s -S $(QEMU_ARGS)

# generate dts from dtb
dtc:
	dtc -o dump.dts -O dts -I dtb dump.dtb

# build and run with qemu
run: build qemu
