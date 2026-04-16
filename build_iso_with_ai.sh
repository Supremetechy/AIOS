#!/bin/bash
# Build AI-OS ISO with embedded AI model
# This extends the base build_iso.sh with AI model integration

set -e

echo "════════════════════════════════════════════════════════════════"
echo "AI-OS ISO Builder with On-Device AI"
echo "════════════════════════════════════════════════════════════════"

# Configuration
MODEL_NAME="gemma-2b-it-q4_k_m.gguf"
MODEL_PATH="models/$MODEL_NAME"
ISO_NAME="aios-ai-$(date +%Y%m%d).iso"
BUILD_DIR="build/iso"
INITRAMFS_DIR="$BUILD_DIR/initramfs"

# Step 1: Build llama.cpp
echo ""
echo "[1/6] Building llama.cpp..."
echo "────────────────────────────────────────────────────────────────"
cd kernel_rs
if [ ! -f "libllama.a" ]; then
    ./build_llama.sh
else
    echo "✓ llama.cpp already built (libllama.a exists)"
fi
cd ..

# Step 2: Build kernel
echo ""
echo "[2/6] Building AI-OS kernel..."
echo "────────────────────────────────────────────────────────────────"
cd kernel_rs
cargo build --release
cd ..

# Step 3: Check for AI model
echo ""
echo "[3/6] Checking AI model..."
echo "────────────────────────────────────────────────────────────────"
if [ ! -f "$MODEL_PATH" ]; then
    echo "⚠  Model not found: $MODEL_PATH"
    echo ""
    echo "Please download the model:"
    echo "  1. Install Hugging Face CLI: pip install huggingface-hub"
    echo "  2. Download model:"
    echo "     cd models"
    echo "     huggingface-cli download google/gemma-2b-it-GGUF $MODEL_NAME --local-dir ."
    echo ""
    echo "Or download manually from:"
    echo "  https://huggingface.co/google/gemma-2b-it-GGUF/blob/main/$MODEL_NAME"
    echo ""
    read -p "Continue without AI model? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    INCLUDE_MODEL=false
else
    MODEL_SIZE=$(du -h "$MODEL_PATH" | cut -f1)
    echo "✓ Found model: $MODEL_PATH ($MODEL_SIZE)"
    INCLUDE_MODEL=true
fi

# Step 4: Create initramfs with model
echo ""
echo "[4/6] Creating initramfs..."
echo "────────────────────────────────────────────────────────────────"
mkdir -p "$INITRAMFS_DIR"/{models,bin,lib,etc,dev,proc,sys}

# Copy kernel
cp kernel_rs/target/release/aios_kernel "$INITRAMFS_DIR/bin/"

# Copy model if available
if [ "$INCLUDE_MODEL" = true ]; then
    echo "Copying AI model to initramfs..."
    cp "$MODEL_PATH" "$INITRAMFS_DIR/models/"
    echo "✓ Model embedded in initramfs"
else
    echo "⚠ Building without AI model"
fi

# Create init script
cat > "$INITRAMFS_DIR/init" << 'EOF'
#!/bin/sh
# AI-OS Init Script

mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev

echo "Starting AI-OS..."
exec /bin/aios_kernel
EOF

chmod +x "$INITRAMFS_DIR/init"

# Pack initramfs
echo "Packing initramfs..."
cd "$INITRAMFS_DIR"
find . | cpio -o -H newc | gzip > ../initramfs.img
cd ../../..

# Step 5: Create GRUB configuration
echo ""
echo "[5/6] Configuring bootloader..."
echo "────────────────────────────────────────────────────────────────"
mkdir -p "$BUILD_DIR"/{boot/grub,iso}

cp boot/grub.cfg "$BUILD_DIR/boot/grub/"

# Update GRUB config for AI
cat > "$BUILD_DIR/boot/grub/grub.cfg" << EOF
set timeout=5
set default=0

menuentry "AI-OS (with On-Device AI)" {
    multiboot2 /boot/aios_kernel
    module2 /boot/initramfs.img
}

menuentry "AI-OS (Safe Mode - No AI)" {
    multiboot2 /boot/aios_kernel noai
    module2 /boot/initramfs.img
}
EOF

# Copy kernel and initramfs
cp kernel_rs/target/release/aios_kernel "$BUILD_DIR/boot/"
cp "$BUILD_DIR/initramfs.img" "$BUILD_DIR/boot/"

# Step 6: Generate ISO
echo ""
echo "[6/6] Generating ISO image..."
echo "────────────────────────────────────────────────────────────────"

if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o "$ISO_NAME" "$BUILD_DIR"
elif command -v grub2-mkrescue &> /dev/null; then
    grub2-mkrescue -o "$ISO_NAME" "$BUILD_DIR"
else
    echo "ERROR: grub-mkrescue not found"
    echo "Install with: sudo apt install grub-pc-bin xorriso (Ubuntu/Debian)"
    echo "          or: sudo yum install grub2-tools xorriso (Fedora/RHEL)"
    exit 1
fi

ISO_SIZE=$(du -h "$ISO_NAME" | cut -f1)

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ ISO built successfully!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "  File: $ISO_NAME"
echo "  Size: $ISO_SIZE"
echo ""
if [ "$INCLUDE_MODEL" = true ]; then
    echo "  AI Model: Included ($MODEL_SIZE)"
    echo "  Features: Full on-device AI capabilities"
else
    echo "  AI Model: Not included"
    echo "  Features: Limited (stub mode)"
fi
echo ""
echo "Test in QEMU:"
echo "  ./run_qemu.sh $ISO_NAME"
echo ""
echo "Or burn to USB:"
echo "  sudo dd if=$ISO_NAME of=/dev/sdX bs=4M status=progress"
echo ""
echo "════════════════════════════════════════════════════════════════"
