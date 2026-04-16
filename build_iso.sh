#!/bin/bash
# AI-OS ISO Builder Script
# Creates a bootable ISO image using GRUB

set -e

echo "========================================="
echo "AI-OS ISO Builder"
echo "========================================="

# Check for required tools
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "ERROR: $1 is not installed"
        echo "Please install it using:"
        case $1 in
            grub-mkrescue)
                echo "  macOS: brew install grub"
                echo "  Ubuntu/Debian: sudo apt-get install grub-pc-bin grub-common xorriso"
                echo "  Arch: sudo pacman -S grub xorriso"
                ;;
            xorriso)
                echo "  macOS: brew install xorriso"
                echo "  Ubuntu/Debian: sudo apt-get install xorriso"
                echo "  Arch: sudo pacman -S xorriso"
                ;;
            cargo)
                echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                ;;
        esac
        exit 1
    fi
}

echo "Checking required tools..."
check_tool grub-mkrescue
check_tool xorriso
check_tool cargo

# Create build directory structure
echo "Creating build directories..."
rm -rf build/iso
mkdir -p build/iso/boot/grub
mkdir -p build/iso/boot/modules

# Build Rust kernel
echo "Building Rust kernel..."
cargo build --release --manifest-path kernel_rs/Cargo.toml

# Copy kernel binary
echo "Copying kernel binary..."
cp kernel_rs/target/x86_64-aios/release/aios_kernel build/iso/boot/aios_kernel.bin

# Copy GRUB configuration
echo "Copying GRUB configuration..."
cp grub.cfg build/iso/boot/grub/grub.cfg

# Create ISO
echo "Creating bootable ISO..."
grub-mkrescue -o aios.iso build/iso

echo ""
echo "========================================="
echo "Build complete!"
echo "========================================="
echo "ISO file: aios.iso"
echo "Size: $(du -h aios.iso | cut -f1)"
echo ""
echo "To test the ISO:"
echo "  ./run_qemu.sh"
echo ""
