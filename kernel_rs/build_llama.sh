#!/bin/bash
# Build llama.cpp as a static library for linking with AI-OS kernel

set -e

LLAMA_DIR="llama.cpp"
BUILD_DIR="$LLAMA_DIR/build"
LIB_OUTPUT="libllama.a"

echo "════════════════════════════════════════════════════════════════"
echo "Building llama.cpp for AI-OS kernel"
echo "════════════════════════════════════════════════════════════════"

# Check if llama.cpp exists
if [ ! -d "$LLAMA_DIR" ]; then
    echo "[BUILD] Cloning llama.cpp..."
    git clone https://github.com/ggerganov/llama.cpp.git
    cd "$LLAMA_DIR"
    # Pin to a stable version
    git checkout b2101
else
    echo "[BUILD] llama.cpp directory exists"
    cd "$LLAMA_DIR"
fi

# Create build directory
echo "[BUILD] Creating build directory..."
mkdir -p build
cd build

# Configure with CMake for static library
echo "[BUILD] Configuring llama.cpp..."
cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=OFF \
    -DLLAMA_STATIC=ON \
    -DLLAMA_NATIVE=OFF \
    -DLLAMA_BUILD_TESTS=OFF \
    -DLLAMA_BUILD_EXAMPLES=OFF \
    -DLLAMA_METAL=OFF \
    -DLLAMA_CUBLAS=OFF \
    -DLLAMA_HIPBLAS=OFF

# Build
echo "[BUILD] Building llama.cpp (this may take a few minutes)..."
make -j$(nproc)

# Copy library to kernel directory
echo "[BUILD] Copying library to kernel_rs/..."
cp libllama.a ../../

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ llama.cpp built successfully!"
echo "  Library: kernel_rs/libllama.a"
echo "  Size: $(du -h ../../libllama.a | cut -f1)"
echo "════════════════════════════════════════════════════════════════"
