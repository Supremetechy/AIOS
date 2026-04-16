#!/bin/bash
# QEMU Debug Mode - runs with GDB support

set -e

ISO_FILE="aios.iso"
KERNEL="kernel_rs/target/x86_64-aios/release/aios_kernel"
GDB_PORT="1234"

echo "========================================="
echo "AI-OS QEMU Debug Mode"
echo "========================================="
echo "GDB will listen on port $GDB_PORT"
echo ""
echo "To attach GDB, run in another terminal:"
echo "  gdb $KERNEL"
echo "  (gdb) target remote localhost:$GDB_PORT"
echo "  (gdb) continue"
echo ""
echo "========================================="

# Check which file to use
if [ -f "$ISO_FILE" ]; then
    echo "Using ISO: $ISO_FILE"
    qemu-system-x86_64 \
        -cdrom "$ISO_FILE" \
        -m 2G \
        -smp 2 \
        -boot d \
        -serial stdio \
        -display none \
        -s -S \
        -no-reboot
elif [ -f "$KERNEL" ]; then
    echo "Using kernel: $KERNEL"
    qemu-system-x86_64 \
        -kernel "$KERNEL" \
        -m 512M \
        -serial stdio \
        -display none \
        -s -S \
        -no-reboot
else
    echo "ERROR: Neither ISO nor kernel binary found"
    echo "Please run ./build_iso.sh first"
    exit 1
fi
