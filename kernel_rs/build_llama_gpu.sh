#!/bin/bash
# Build llama.cpp with GPU acceleration support

set -e

LLAMA_DIR="llama.cpp"
BUILD_DIR="$LLAMA_DIR/build"

echo "════════════════════════════════════════════════════════════════"
echo "Building llama.cpp with GPU Acceleration"
echo "════════════════════════════════════════════════════════════════"

# Detect GPU
GPU_TYPE="none"
if command -v nvidia-smi &> /dev/null; then
    echo "[DETECT] NVIDIA GPU detected"
    GPU_TYPE="cuda"
elif command -v rocm-smi &> /dev/null; then
    echo "[DETECT] AMD GPU detected"
    GPU_TYPE="rocm"
elif [ "$(uname)" = "Darwin" ] && [ "$(uname -m)" = "arm64" ]; then
    echo "[DETECT] Apple Silicon detected"
    GPU_TYPE="metal"
else
    echo "[DETECT] No GPU detected - building CPU-only version"
    GPU_TYPE="none"
fi

# Clone llama.cpp if needed
if [ ! -d "$LLAMA_DIR" ]; then
    echo "[BUILD] Cloning llama.cpp..."
    git clone https://github.com/ggerganov/llama.cpp.git
    cd "$LLAMA_DIR"
    git checkout b2101
else
    echo "[BUILD] llama.cpp directory exists"
    cd "$LLAMA_DIR"
fi

# Clean previous build
rm -rf build
mkdir -p build
cd build

# Configure based on GPU type
echo "[BUILD] Configuring for $GPU_TYPE..."

case $GPU_TYPE in
    cuda)
        cmake .. \
            -DCMAKE_BUILD_TYPE=Release \
            -DBUILD_SHARED_LIBS=OFF \
            -DLLAMA_STATIC=ON \
            -DLLAMA_CUBLAS=ON \
            -DCMAKE_CUDA_ARCHITECTURES=native \
            -DLLAMA_BUILD_TESTS=OFF \
            -DLLAMA_BUILD_EXAMPLES=OFF
        ;;
    rocm)
        cmake .. \
            -DCMAKE_BUILD_TYPE=Release \
            -DBUILD_SHARED_LIBS=OFF \
            -DLLAMA_STATIC=ON \
            -DLLAMA_HIPBLAS=ON \
            -DCMAKE_C_COMPILER=hipcc \
            -DCMAKE_CXX_COMPILER=hipcc \
            -DLLAMA_BUILD_TESTS=OFF \
            -DLLAMA_BUILD_EXAMPLES=OFF
        ;;
    metal)
        cmake .. \
            -DCMAKE_BUILD_TYPE=Release \
            -DBUILD_SHARED_LIBS=OFF \
            -DLLAMA_STATIC=ON \
            -DLLAMA_METAL=ON \
            -DLLAMA_BUILD_TESTS=OFF \
            -DLLAMA_BUILD_EXAMPLES=OFF
        ;;
    *)
        cmake .. \
            -DCMAKE_BUILD_TYPE=Release \
            -DBUILD_SHARED_LIBS=OFF \
            -DLLAMA_STATIC=ON \
            -DLLAMA_NATIVE=OFF \
            -DLLAMA_BUILD_TESTS=OFF \
            -DLLAMA_BUILD_EXAMPLES=OFF
        ;;
esac

# Build
echo "[BUILD] Compiling llama.cpp..."
make -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

# Copy library
echo "[BUILD] Installing library..."
cp libllama.a ../../

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ llama.cpp built successfully with $GPU_TYPE support!"
echo "  Library: kernel_rs/libllama.a"
echo "  Size: $(du -h ../../libllama.a | cut -f1)"
echo "════════════════════════════════════════════════════════════════"
