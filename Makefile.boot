# AI-OS Boot & QEMU Makefile
# Convenient targets for building and testing

.PHONY: all kernel iso clean test test-simple test-debug install help

# Default target
all: iso

# Build the Rust kernel
kernel:
	@echo "Building Rust kernel..."
	cd kernel_rs && cargo build --release

# Build bootable ISO
iso: kernel
	@echo "Building bootable ISO..."
	./build_iso.sh

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf build/iso
	rm -f aios.iso
	cd kernel_rs && cargo clean

# Test with QEMU (full ISO)
test: iso
	@echo "Testing with QEMU..."
	./run_qemu.sh

# Quick test (kernel only, no ISO)
test-simple: kernel
	@echo "Quick testing with QEMU..."
	./test_qemu_simple.sh

# Test with GDB debugging
test-debug: iso
	@echo "Starting QEMU in debug mode..."
	@echo "Connect GDB in another terminal with:"
	@echo "  gdb kernel_rs/target/x86_64-aios/release/aios_kernel"
	@echo "  (gdb) target remote localhost:1234"
	./qemu_debug.sh

# Install GRUB and QEMU (platform detection)
install:
	@echo "Detecting platform..."
	@if [ "$$(uname)" = "Darwin" ]; then \
		echo "Installing on macOS..."; \
		brew install qemu grub xorriso; \
	elif [ -f /etc/debian_version ]; then \
		echo "Installing on Debian/Ubuntu..."; \
		sudo apt-get update && sudo apt-get install -y qemu-system-x86 grub-pc-bin grub-common xorriso mtools; \
	elif [ -f /etc/arch-release ]; then \
		echo "Installing on Arch Linux..."; \
		sudo pacman -S qemu grub xorriso; \
	elif [ -f /etc/fedora-release ]; then \
		echo "Installing on Fedora..."; \
		sudo dnf install -y qemu-system-x86 grub2-tools-extra xorriso mtools; \
	else \
		echo "Unknown platform. Please install manually."; \
		echo "See INSTALL_QEMU.md for instructions."; \
		exit 1; \
	fi

# Help target
help:
	@echo "AI-OS Build & Test Targets"
	@echo "============================"
	@echo ""
	@echo "Targets:"
	@echo "  make kernel       - Build Rust kernel"
	@echo "  make iso          - Build bootable ISO (default)"
	@echo "  make test         - Test ISO in QEMU"
	@echo "  make test-simple  - Quick kernel test (no ISO)"
	@echo "  make test-debug   - Run QEMU with GDB support"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make install      - Install QEMU and GRUB"
	@echo "  make help         - Show this help"
	@echo ""
	@echo "Quick Start:"
	@echo "  make install      # First time only"
	@echo "  make test         # Build and test"
	@echo ""
	@echo "Development Loop:"
	@echo "  1. Edit code in kernel_rs/src/"
	@echo "  2. make test-simple"
	@echo "  3. Repeat"
	@echo ""
