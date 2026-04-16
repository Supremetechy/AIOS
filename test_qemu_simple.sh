#!/bin/bash
# Simple QEMU test without ISO - tests kernel binary directly

set -e

KERNEL="kernel_rs/target/x86_64-aios/release/aios_kernel"

echo "========================================="
echo "AI-OS Simple QEMU Test"
echo "========================================="

# Build kernel if it doesn't exist
if [ ! -f "$KERNEL" ]; then
    echo "Kernel not found, building..."
    cargo build --release --manifest-path kernel_rs/Cargo.toml
fi

echo "Testing kernel binary directly..."
echo "Press Ctrl+A then X to exit QEMU"
echo "========================================="
echo ""

qemu-system-x86_64 \
    -kernel "$KERNEL" \
    -m 512M \
    -serial stdio \
    -display none \
    -no-reboot \
    -enable-kvm 2>/dev/null || \
    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -m 512M \
        -serial stdio \
        -display none \
        -no-reboot
