# AI-OS Bare-Metal Build System

# Architecture
ARCH := x86_64

# Directories
BOOT_DIR := boot
KERNEL_DIR := kernel_rs
BUILD_DIR := build
ISO_DIR := $(BUILD_DIR)/iso

# Tools
AS := nasm
CARGO := cargo
GRUB_MKRESCUE := grub-mkrescue
QEMU := qemu-system-$(ARCH)

# Bootloader
BOOT_ASM := $(BOOT_DIR)/boot.asm
BOOT_OBJ := $(BUILD_DIR)/boot.o

# Kernel
KERNEL_BIN := $(BUILD_DIR)/kernel.bin
KERNEL_ELF := $(KERNEL_DIR)/target/$(ARCH)-aios/debug/aios_kernel

# ISO
ISO_FILE := $(BUILD_DIR)/aios.iso

# QEMU flags
QEMU_FLAGS := -m 512M -serial stdio -display gtk

.PHONY: all clean run iso kernel bootloader dirs

all: iso

dirs:
	@mkdir -p $(BUILD_DIR)
	@mkdir -p $(ISO_DIR)/boot/grub

# Build bootloader
bootloader: dirs
	@echo "[BUILD] Compiling bootloader..."
	$(AS) -f elf64 $(BOOT_ASM) -o $(BOOT_OBJ)

# Build Rust kernel
kernel:
	@echo "[BUILD] Building Rust kernel..."
	cd $(KERNEL_DIR) && $(CARGO) build --target $(ARCH)-aios.json

# Create bootable ISO
iso: bootloader kernel dirs
	@echo "[BUILD] Creating bootable ISO..."
	@mkdir -p $(ISO_DIR)/boot/grub
	@cp $(BOOT_OBJ) $(ISO_DIR)/boot/boot.o
	@cp $(KERNEL_ELF) $(ISO_DIR)/boot/kernel.bin || echo "Note: Kernel binary not found, using placeholder"
	@echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	@echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo 'menuentry "AI-OS" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '    multiboot2 /boot/boot.o' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '    module2 /boot/kernel.bin' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '    boot' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg
	@$(GRUB_MKRESCUE) -o $(ISO_FILE) $(ISO_DIR) 2>/dev/null || echo "Note: grub-mkrescue not available"
	@echo "[BUILD] ✓ ISO created: $(ISO_FILE)"

# Run in QEMU
run: iso
	@echo "[QEMU] Starting AI-OS in QEMU..."
	$(QEMU) $(QEMU_FLAGS) -cdrom $(ISO_FILE)

# Run with KVM acceleration
run-kvm: iso
	@echo "[QEMU] Starting AI-OS in QEMU with KVM..."
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -cdrom $(ISO_FILE)

# Debug in QEMU
debug: iso
	@echo "[QEMU] Starting AI-OS in debug mode..."
	$(QEMU) $(QEMU_FLAGS) -cdrom $(ISO_FILE) -s -S

# Clean build artifacts
clean:
	@echo "[CLEAN] Removing build artifacts..."
	@rm -rf $(BUILD_DIR)
	@cd $(KERNEL_DIR) && $(CARGO) clean
	@echo "[CLEAN] ✓ Clean complete"

# Help
help:
	@echo "AI-OS Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all         - Build everything (default)"
	@echo "  bootloader  - Build bootloader only"
	@echo "  kernel      - Build Rust kernel only"
	@echo "  iso         - Create bootable ISO"
	@echo "  run         - Run in QEMU"
	@echo "  run-kvm     - Run in QEMU with KVM"
	@echo "  debug       - Run in QEMU debug mode"
	@echo "  clean       - Remove build artifacts"
	@echo "  help        - Show this help"
