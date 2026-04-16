#!/bin/bash
# Production deployment script for AI-OS
# Builds optimized release with all features enabled

set -e

echo "════════════════════════════════════════════════════════════════"
echo "AI-OS Production Deployment Builder"
echo "════════════════════════════════════════════════════════════════"

# Configuration
VERSION=$(date +%Y.%m.%d)
BUILD_DIR="build/production"
RELEASE_DIR="release/aios-${VERSION}"

# Check prerequisites
echo ""
echo "[1/8] Checking prerequisites..."
echo "────────────────────────────────────────────────────────────────"

command -v cargo >/dev/null 2>&1 || { echo "✗ Rust/Cargo not found"; exit 1; }
command -v cmake >/dev/null 2>&1 || { echo "✗ CMake not found"; exit 1; }
command -v grub-mkrescue >/dev/null 2>&1 || command -v grub2-mkrescue >/dev/null 2>&1 || { echo "✗ GRUB not found"; exit 1; }

echo "✓ Prerequisites satisfied"

# Clean previous builds
echo ""
echo "[2/8] Cleaning previous builds..."
echo "────────────────────────────────────────────────────────────────"
rm -rf "$BUILD_DIR"
rm -rf "$RELEASE_DIR"
mkdir -p "$BUILD_DIR"
mkdir -p "$RELEASE_DIR"

# Build llama.cpp with GPU
echo ""
echo "[3/8] Building llama.cpp with GPU support..."
echo "────────────────────────────────────────────────────────────────"
cd kernel_rs
./build_llama_gpu.sh
cd ..

# Build kernel in release mode
echo ""
echo "[4/8] Building AI-OS kernel (release mode)..."
echo "────────────────────────────────────────────────────────────────"
cd kernel_rs
RUSTFLAGS="-C opt-level=3 -C lto=fat" cargo build --release
cd ..

echo "✓ Kernel built successfully"

# Check for AI model
echo ""
echo "[5/8] Verifying AI model..."
echo "────────────────────────────────────────────────────────────────"

MODEL_PATH="models/gemma-2b-it-q4_k_m.gguf"
if [ -f "$MODEL_PATH" ]; then
    MODEL_SIZE=$(du -h "$MODEL_PATH" | cut -f1)
    echo "✓ Model found: $MODEL_SIZE"
else
    echo "⚠  Model not found at $MODEL_PATH"
    echo ""
    echo "Download with:"
    echo "  cd models"
    echo "  huggingface-cli download google/gemma-2b-it-GGUF gemma-2b-it-q4_k_m.gguf --local-dir ."
    echo ""
    read -p "Continue without model? (y/N) " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
fi

# Create bootable ISO
echo ""
echo "[6/8] Creating bootable ISO..."
echo "────────────────────────────────────────────────────────────────"

ISO_DIR="$BUILD_DIR/iso"
mkdir -p "$ISO_DIR"/{boot/grub,models}

# Copy kernel
cp kernel_rs/target/release/aios_kernel "$ISO_DIR/boot/"

# Copy model if available
if [ -f "$MODEL_PATH" ]; then
    cp "$MODEL_PATH" "$ISO_DIR/models/"
fi

# Create GRUB config
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'GRUBEOF'
set timeout=3
set default=0

menuentry "AI-OS Production" {
    multiboot2 /boot/aios_kernel
}

menuentry "AI-OS Safe Mode" {
    multiboot2 /boot/aios_kernel --safe-mode
}
GRUBEOF

# Generate ISO
if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o "$RELEASE_DIR/aios-${VERSION}.iso" "$ISO_DIR"
else
    grub2-mkrescue -o "$RELEASE_DIR/aios-${VERSION}.iso" "$ISO_DIR"
fi

ISO_SIZE=$(du -h "$RELEASE_DIR/aios-${VERSION}.iso" | cut -f1)

# Create checksums
echo ""
echo "[7/8] Generating checksums..."
echo "────────────────────────────────────────────────────────────────"
cd "$RELEASE_DIR"
sha256sum "aios-${VERSION}.iso" > "aios-${VERSION}.iso.sha256"
md5sum "aios-${VERSION}.iso" > "aios-${VERSION}.iso.md5"
cd ../..

# Create release notes
echo ""
echo "[8/8] Creating release package..."
echo "────────────────────────────────────────────────────────────────"

cat > "$RELEASE_DIR/RELEASE_NOTES.txt" << EOF
AI-OS Production Release ${VERSION}
═══════════════════════════════════════════════════════════════

Build Date: $(date)
Version: ${VERSION}

FEATURES
────────────────────────────────────────────────────────────────
✓ On-Device AI (Gemma 2B with GPU acceleration)
✓ Cloud AI (Gemini, OpenAI, Anthropic)
✓ Full TCP/IP Network Stack (smoltcp)
✓ TLS/HTTPS Support (rustls)
✓ Intel e1000 NIC Driver
✓ DHCP Auto-Configuration
✓ Real DNS Resolution
✓ Multi-Provider Failover
✓ Hybrid AI Mode (4 modes)

SYSTEM REQUIREMENTS
────────────────────────────────────────────────────────────────
Minimum:
- CPU: x86_64 (4 cores)
- RAM: 6 GB
- Storage: 10 GB
- Network: Optional (for cloud AI)

Recommended:
- CPU: x86_64 (8+ cores)
- RAM: 16 GB
- GPU: NVIDIA RTX 3070+ or AMD RX 6000+
- Storage: 20 GB
- Network: 100 Mbps+

INSTALLATION
────────────────────────────────────────────────────────────────
1. Verify checksum:
   sha256sum -c aios-${VERSION}.iso.sha256

2. Burn to USB:
   sudo dd if=aios-${VERSION}.iso of=/dev/sdX bs=4M status=progress

3. Or test in QEMU:
   qemu-system-x86_64 -cdrom aios-${VERSION}.iso -m 8G

CONFIGURATION
────────────────────────────────────────────────────────────────
API Keys (optional for cloud AI):
- Gemini: https://aistudio.google.com/app/apikey
- OpenAI: https://platform.openai.com/api-keys
- Anthropic: https://console.anthropic.com/

DOCUMENTATION
────────────────────────────────────────────────────────────────
- Quick Start: QUICKSTART_AI.md
- GPU Setup: GPU_ACCELERATION_GUIDE.md
- Cloud APIs: CLOUD_API_GUIDE.md
- Network: NETWORK_COMPLETE.md
- Full Guide: COMPLETE_IMPLEMENTATION_SUMMARY.md

SUPPORT
────────────────────────────────────────────────────────────────
Issues: https://github.com/yourusername/aios/issues
Docs: https://docs.aios.dev

CHECKSUMS
────────────────────────────────────────────────────────────────
SHA256: $(cat aios-${VERSION}.iso.sha256 | cut -d' ' -f1)
MD5:    $(cat aios-${VERSION}.iso.md5 | cut -d' ' -f1)

═══════════════════════════════════════════════════════════════
EOF

# Copy documentation
cp QUICKSTART_AI.md "$RELEASE_DIR/" 2>/dev/null || true
cp GPU_ACCELERATION_GUIDE.md "$RELEASE_DIR/" 2>/dev/null || true
cp CLOUD_API_GUIDE.md "$RELEASE_DIR/" 2>/dev/null || true
cp NETWORK_COMPLETE.md "$RELEASE_DIR/" 2>/dev/null || true

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ Production build complete!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Release Directory: $RELEASE_DIR"
echo "ISO File: aios-${VERSION}.iso ($ISO_SIZE)"
echo ""
echo "Contents:"
ls -lh "$RELEASE_DIR"
echo ""
echo "Test with:"
echo "  ./test_qemu_network.sh $RELEASE_DIR/aios-${VERSION}.iso"
echo ""
echo "Deploy to USB:"
echo "  sudo dd if=$RELEASE_DIR/aios-${VERSION}.iso of=/dev/sdX bs=4M status=progress"
echo ""
echo "════════════════════════════════════════════════════════════════"
