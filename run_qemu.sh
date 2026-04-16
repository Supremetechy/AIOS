#!/bin/bash
# AI-OS QEMU Test Runner
# Runs the AI-OS ISO in QEMU for testing

set -e

ISO_FILE="aios.iso"
MEMORY="2G"
CPUS="4"
DISPLAY_MODE="gtk"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--memory)
            MEMORY="$2"
            shift 2
            ;;
        -c|--cpus)
            CPUS="$2"
            shift 2
            ;;
        --headless)
            DISPLAY_MODE="none"
            shift
            ;;
        --vnc)
            DISPLAY_MODE="vnc=:0"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -m, --memory SIZE    Set memory size (default: 2G)"
            echo "  -c, --cpus COUNT     Set CPU count (default: 4)"
            echo "  --headless           Run without display"
            echo "  --vnc                Use VNC display on :5900"
            echo "  -h, --help           Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use -h for help"
            exit 1
            ;;
    esac
done

# Check if ISO exists
if [ ! -f "$ISO_FILE" ]; then
    echo "ERROR: ISO file not found: $ISO_FILE"
    echo "Please run ./build_iso.sh first"
    exit 1
fi

# Check for QEMU
if ! command -v qemu-system-x86_64 &> /dev/null; then
    echo "ERROR: qemu-system-x86_64 is not installed"
    echo "Please install it using:"
    echo "  macOS: brew install qemu"
    echo "  Ubuntu/Debian: sudo apt-get install qemu-system-x86"
    echo "  Arch: sudo pacman -S qemu"
    exit 1
fi

echo "========================================="
echo "AI-OS QEMU Test Runner"
echo "========================================="
echo "ISO: $ISO_FILE"
echo "Memory: $MEMORY"
echo "CPUs: $CPUS"
echo "Display: $DISPLAY_MODE"
echo ""
echo "Starting QEMU..."
echo "Press Ctrl+Alt+G to release mouse/keyboard"
echo "========================================="
echo ""

# Run QEMU
qemu-system-x86_64 \
    -cdrom "$ISO_FILE" \
    -m "$MEMORY" \
    -smp "$CPUS" \
    -boot d \
    -display "$DISPLAY_MODE" \
    -serial stdio \
    -enable-kvm 2>/dev/null || \
    qemu-system-x86_64 \
        -cdrom "$ISO_FILE" \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -boot d \
        -display "$DISPLAY_MODE" \
        -serial stdio

echo ""
echo "QEMU session ended."
