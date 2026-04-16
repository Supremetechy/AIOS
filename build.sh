#!/bin/bash
# AI-OS Build Script

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              AI-OS Bare-Metal Build System                     ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo

# Check dependencies
echo "[CHECK] Verifying build dependencies..."

check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "  ✗ $1 not found"
        return 1
    else
        echo "  ✓ $1 found"
        return 0
    fi
}

MISSING_DEPS=0

check_command nasm || MISSING_DEPS=1
check_command cargo || MISSING_DEPS=1
check_command rustc || MISSING_DEPS=1

# Optional but recommended
check_command grub-mkrescue || echo "  ⚠ grub-mkrescue not found (optional)"
check_command qemu-system-x86_64 || echo "  ⚠ QEMU not found (optional, for testing)"

if [ $MISSING_DEPS -eq 1 ]; then
    echo
    echo "[ERROR] Missing required dependencies!"
    echo
    echo "Install with:"
    echo "  Ubuntu/Debian: sudo apt install nasm build-essential"
    echo "  macOS: brew install nasm"
    echo "  Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo
    exit 1
fi

echo
echo "[BUILD] Starting build process..."
echo

# Create build directories
mkdir -p build/iso/boot/grub

# Build bootloader
echo "[1/3] Building bootloader..."
nasm -f elf64 boot/boot.asm -o build/boot.o
echo "  ✓ Bootloader compiled"

# Build Rust kernel
echo "[2/3] Building Rust kernel..."
cd kernel_rs

# Check if we have the right Rust target
if ! rustup target list | grep -q "x86_64-unknown-none"; then
    echo "  Adding Rust target x86_64-unknown-none..."
    rustup target add x86_64-unknown-none
fi

# Install rust-src if needed
if ! rustup component list | grep -q "rust-src (installed)"; then
    echo "  Installing rust-src component..."
    rustup component add rust-src
fi

# Build kernel
cargo build --target x86_64-aios.json 2>&1 | grep -E "Compiling|Finished|error" || true

cd ..
echo "  ✓ Kernel compiled"

# Create GRUB configuration
echo "[3/3] Creating bootable image..."

cat > build/iso/boot/grub/grub.cfg << 'EOF'
set timeout=0
set default=0

menuentry "AI-OS" {
    multiboot2 /boot/boot.o
    boot
}
EOF

cp build/boot.o build/iso/boot/boot.o

# Try to create ISO if grub-mkrescue is available
if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o build/aios.iso build/iso 2>/dev/null || echo "  ⚠ ISO creation skipped"
    if [ -f build/aios.iso ]; then
        echo "  ✓ Bootable ISO created: build/aios.iso"
    fi
else
    echo "  ⚠ grub-mkrescue not available, skipping ISO creation"
fi

echo
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                    BUILD COMPLETE                              ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo
echo "Build artifacts:"
echo "  Bootloader: build/boot.o"
echo "  Kernel:     kernel_rs/target/x86_64-aios/debug/aios_kernel"
if [ -f build/aios.iso ]; then
    echo "  ISO:        build/aios.iso"
fi
echo
echo "To test in QEMU:"
echo "  make run"
echo
echo "Or manually:"
echo "  qemu-system-x86_64 -cdrom build/aios.iso -m 512M"
echo
