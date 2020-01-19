MODE ?= debug
EFI := target/x86_64-unknown-uefi/$(MODE)/uefi.efi
OVMF := OVMF.fd
ESP := target/esp
BUILD_ARGS := -Z build-std=core,alloc
QEMU_ARGS := -net none -nographic

ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

build:
	cargo build $(BUILD_ARGS)

run:
	mkdir -p $(ESP)/EFI/Boot
	cp $(EFI) $(ESP)/EFI/Boot/BootX64.efi
	qemu-system-x86_64 \
		-bios ${OVMF} \
		-drive format=raw,file=fat:rw:${ESP} \
		$(QEMU_ARGS)
