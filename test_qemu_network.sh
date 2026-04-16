#!/bin/bash
# QEMU network testing script for AI-OS
# Tests network stack, DHCP, DNS, and cloud API connectivity

set -e

ISO_FILE="${1:-aios-ai-*.iso}"

if [ ! -f "$ISO_FILE" ]; then
    echo "Error: ISO file not found: $ISO_FILE"
    echo "Usage: $0 <iso-file>"
    exit 1
fi

echo "════════════════════════════════════════════════════════════════"
echo "AI-OS QEMU Network Testing"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "ISO: $ISO_FILE"
echo "Testing: Network Stack, DHCP, DNS, TLS, Cloud APIs"
echo ""

# Check if KVM is available
if [ -e /dev/kvm ]; then
    KVM_FLAGS="-enable-kvm -cpu host"
    echo "✓ KVM acceleration enabled"
else
    KVM_FLAGS=""
    echo "⚠  KVM not available (will be slower)"
fi

# Detect GPU
GPU_FLAGS=""
if command -v nvidia-smi &> /dev/null; then
    echo "✓ NVIDIA GPU detected"
    GPU_FLAGS="-vga std"
elif lspci | grep -i "VGA.*AMD" &> /dev/null; then
    echo "✓ AMD GPU detected"
    GPU_FLAGS="-vga std"
fi

echo ""
echo "Starting QEMU with network configuration..."
echo "────────────────────────────────────────────────────────────────"
echo "Network Mode: User (NAT)"
echo "NIC: Intel e1000"
echo "DHCP: Enabled (10.0.2.2/24)"
echo "DNS: Host DNS forwarding"
echo "Internet: Full access"
echo ""
echo "Press Ctrl+Alt+G to release mouse"
echo "Press Ctrl+Alt+2 for QEMU monitor"
echo "Press Ctrl+Alt+1 to return to console"
echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Run QEMU with full network support
qemu-system-x86_64 \
    -cdrom "$ISO_FILE" \
    -m 8G \
    -smp 4 \
    $KVM_FLAGS \
    $GPU_FLAGS \
    -device e1000,netdev=net0,mac=52:54:00:12:34:56 \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -serial stdio \
    -monitor telnet:127.0.0.1:55555,server,nowait \
    -name "AI-OS Network Test"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "QEMU session ended"
echo "════════════════════════════════════════════════════════════════"
